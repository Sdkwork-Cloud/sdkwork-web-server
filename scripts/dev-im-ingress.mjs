#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createServer as createHttpServer, request as httpRequest } from 'node:http';
import { createServer as createHttpsServer } from 'node:https';
import { connect } from 'node:net';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  loadImDevConfig,
  resolveImDevConfigPath,
} from './lib/im-dev-config.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);

export function isMobileUserAgent(userAgent, mobileUserAgentTokens) {
  const normalizedUserAgent = String(userAgent ?? '').toLowerCase();
  return mobileUserAgentTokens.some((token) => normalizedUserAgent.includes(token.toLowerCase()));
}

export function selectImDevTarget(userAgent, settings) {
  return isMobileUserAgent(userAgent, settings.mobileUserAgentTokens)
    ? settings.targets.h5
    : settings.targets.pc;
}

export function requestPathMatchesPrefix(requestUrl, pathPrefixes) {
  let pathname;
  try {
    pathname = new URL(requestUrl ?? '/', 'http://sdkwork.local').pathname;
  } catch {
    return false;
  }
  return pathPrefixes.some((prefix) => {
    if (prefix.endsWith('/')) {
      return pathname.startsWith(prefix);
    }
    return pathname === prefix || pathname.startsWith(`${prefix}/`);
  });
}

function ensureDevelopmentCertificate(certificateConfig, publicHost) {
  if (certificateConfig.mode === 'files') {
    if (!existsSync(certificateConfig.certificateFile) || !existsSync(certificateConfig.privateKeyFile)) {
      throw new Error('configured HTTPS certificate or private key file does not exist');
    }
    return certificateConfig;
  }
  const certificateDirectory = certificateConfig.directory;
  const certificateFile = path.join(certificateDirectory, `${publicHost}.pem`);
  const privateKeyFile = path.join(certificateDirectory, `${publicHost}-key.pem`);
  if (existsSync(certificateFile) && existsSync(privateKeyFile)) {
    return { certificateFile, privateKeyFile };
  }

  mkdirSync(certificateDirectory, { recursive: true });
  const mkcert = spawnSync(
    'mkcert',
    ['-cert-file', certificateFile, '-key-file', privateKeyFile, publicHost, 'localhost', '127.0.0.1', '::1'],
    { encoding: 'utf8', windowsHide: true },
  );
  if (mkcert.status === 0) {
    return { certificateFile, privateKeyFile };
  }

  const openssl = spawnSync(
    'openssl',
    [
      'req', '-x509', '-newkey', 'rsa:2048', '-sha256', '-nodes', '-days', '30',
      '-subj', `/CN=${publicHost}`,
      '-addext', `subjectAltName=DNS:${publicHost},DNS:localhost,IP:127.0.0.1,IP:::1`,
      '-keyout', privateKeyFile,
      '-out', certificateFile,
    ],
    { encoding: 'utf8', windowsHide: true },
  );
  if (openssl.status !== 0) {
    throw new Error(
      `cannot create localhost development certificate: ${openssl.stderr || mkcert.stderr}`,
    );
  }
  process.stderr.write(
    '[sdkwork-web] mkcert was unavailable; using an untrusted 30-day localhost certificate\n',
  );
  return { certificateFile, privateKeyFile };
}

function forwardedHeaders(headers, target, protocol) {
  return {
    ...headers,
    host: `${target.host}:${target.port}`,
    'x-forwarded-host': headers.host ?? 'localhost',
    'x-forwarded-proto': protocol,
  };
}

function writeText(response, statusCode, body, headers = {}) {
  response.writeHead(statusCode, {
    'content-type': 'text/plain; charset=utf-8',
    'content-length': Buffer.byteLength(body),
    ...headers,
  });
  response.end(body);
}

function proxyHttpRequest(request, response, target, protocol, unavailableMessage) {
  const upstream = httpRequest({
    host: target.host,
    port: target.port,
    method: request.method,
    path: request.url,
    headers: forwardedHeaders(request.headers, target, protocol),
  });
  upstream.on('response', (upstreamResponse) => {
    response.writeHead(upstreamResponse.statusCode ?? 502, upstreamResponse.headers);
    upstreamResponse.pipe(response);
  });
  upstream.on('error', () => {
    if (!response.headersSent) {
      writeText(response, 502, unavailableMessage);
    } else {
      response.destroy();
    }
  });
  request.on('aborted', () => upstream.destroy());
  request.pipe(upstream);
}

function proxyWebSocketUpgrade(request, socket, head, target, protocol) {
  const upstream = connect(target.port, target.host);
  upstream.once('connect', () => {
    const headers = forwardedHeaders(request.headers, target, protocol);
    const headerLines = Object.entries(headers).flatMap(([name, value]) => {
      if (Array.isArray(value)) {
        return value.map((entry) => `${name}: ${entry}`);
      }
      return value === undefined ? [] : [`${name}: ${value}`];
    });
    upstream.write(`${request.method} ${request.url} HTTP/${request.httpVersion}\r\n`);
    upstream.write(`${headerLines.join('\r\n')}\r\n\r\n`);
    if (head.length > 0) {
      upstream.write(head);
    }
    socket.pipe(upstream).pipe(socket);
  });
  upstream.on('error', () => socket.destroy());
  socket.on('error', () => upstream.destroy());
}

export function requestHostMatches(hostHeader, publicHost, publicPort) {
  if (typeof hostHeader !== 'string' || !hostHeader) {
    return false;
  }
  try {
    const parsed = new URL(`http://${hostHeader}`);
    const requestPort = parsed.port
      ? Number.parseInt(parsed.port, 10)
      : [80, 443].includes(publicPort) ? publicPort : undefined;
    return parsed.hostname.toLowerCase() === publicHost.toLowerCase()
      && requestPort === publicPort;
  } catch {
    return false;
  }
}

export function createImDevIngressServer({ certificate, privateKey, settings }) {
  const requestHandler = (request, response) => {
    if (!requestHostMatches(request.headers.host, settings.publicHost, settings.publicPort)) {
      writeText(response, 421, 'request Host does not match the configured IM application origin\n');
      return;
    }
    const pathWithoutTrailingSlash = settings.pathPrefix.slice(0, -1);
    if (pathWithoutTrailingSlash && request.url === pathWithoutTrailingSlash) {
      writeText(response, 308, '', { location: settings.pathPrefix });
      return;
    }
    if (!request.url?.startsWith(settings.pathPrefix)) {
      writeText(response, 404, 'route was not found\n');
      return;
    }
    const gatewayRequest = requestPathMatchesPrefix(
      request.url,
      settings.gateway.httpPathPrefixes,
    );
    const target = gatewayRequest
      ? settings.gateway.target
      : selectImDevTarget(request.headers['user-agent'], settings);
    const unavailableMessage = gatewayRequest
      ? 'standalone IM application gateway is not ready\n'
      : 'selected IM development application is not ready\n';
    proxyHttpRequest(request, response, target, settings.protocol, unavailableMessage);
  };
  const server = settings.protocol === 'https'
    ? createHttpsServer({ cert: certificate, key: privateKey }, requestHandler)
    : createHttpServer(requestHandler);
  server.on('upgrade', (request, socket, head) => {
    if (!requestHostMatches(request.headers.host, settings.publicHost, settings.publicPort)) {
      socket.destroy();
      return;
    }
    if (!request.url?.startsWith(settings.pathPrefix)) {
      socket.destroy();
      return;
    }
    const target = requestPathMatchesPrefix(
      request.url,
      settings.gateway.webSocketPathPrefixes,
    )
      ? settings.gateway.target
      : selectImDevTarget(request.headers['user-agent'], settings);
    proxyWebSocketUpgrade(request, socket, head, target, settings.protocol);
  });
  return server;
}

async function run() {
  const configArgumentIndex = process.argv.indexOf('--config');
  const configPath = resolveImDevConfigPath(
    configArgumentIndex >= 0 ? process.argv[configArgumentIndex + 1] : undefined,
  );
  const config = loadImDevConfig(configPath);
  const settings = {
    host: config.listener.bind,
    port: config.application.publicPort,
    protocol: config.application.protocol,
    publicHost: config.application.publicHost,
    publicPort: config.application.publicPort,
    publicUrl: config.application.publicUrl,
    pathPrefix: config.route.pathPrefix,
    mobileUserAgentTokens: config.route.mobileUserAgentTokens,
    targets: {
      pc: config.applications.pc,
      h5: config.applications.h5,
    },
    gateway: {
      target: {
        host: config.deployment.gateway.host,
        port: config.deployment.gateway.port,
      },
      httpPathPrefixes: config.deployment.gateway.httpPathPrefixes,
      webSocketPathPrefixes: config.deployment.gateway.webSocketPathPrefixes,
    },
  };
  const certificateFiles = settings.protocol === 'https'
    ? ensureDevelopmentCertificate(config.listener.certificate, settings.publicHost)
    : undefined;
  const server = createImDevIngressServer({
    certificate: certificateFiles ? readFileSync(certificateFiles.certificateFile) : undefined,
    privateKey: certificateFiles ? readFileSync(certificateFiles.privateKeyFile) : undefined,
    settings,
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(settings.port, settings.host, resolve);
  });
  process.stdout.write(
    `[sdkwork-web] IM dev config: ${config.configPath}\n`
      + `[sdkwork-web] IM app manifest: ${config.application.manifestPath}\n`
      + `[sdkwork-web] IM dev ingress: ${settings.publicUrl}\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().catch((error) => {
    process.stderr.write(`[sdkwork-web] ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
