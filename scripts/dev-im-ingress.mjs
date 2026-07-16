#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createServer } from 'node:https';
import { connect } from 'node:net';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { request as httpRequest } from 'node:http';

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

function ensureDevelopmentCertificate(certificateConfig) {
  if (certificateConfig.mode === 'files') {
    if (!existsSync(certificateConfig.certificateFile) || !existsSync(certificateConfig.privateKeyFile)) {
      throw new Error('configured HTTPS certificate or private key file does not exist');
    }
    return certificateConfig;
  }
  const certificateDirectory = certificateConfig.directory;
  const certificateFile = path.join(certificateDirectory, 'localhost.pem');
  const privateKeyFile = path.join(certificateDirectory, 'localhost-key.pem');
  if (existsSync(certificateFile) && existsSync(privateKeyFile)) {
    return { certificateFile, privateKeyFile };
  }

  mkdirSync(certificateDirectory, { recursive: true });
  const mkcert = spawnSync(
    'mkcert',
    ['-cert-file', certificateFile, '-key-file', privateKeyFile, 'localhost', '127.0.0.1', '::1'],
    { encoding: 'utf8', windowsHide: true },
  );
  if (mkcert.status === 0) {
    return { certificateFile, privateKeyFile };
  }

  const openssl = spawnSync(
    'openssl',
    [
      'req', '-x509', '-newkey', 'rsa:2048', '-sha256', '-nodes', '-days', '30',
      '-subj', '/CN=localhost',
      '-addext', 'subjectAltName=DNS:localhost,IP:127.0.0.1,IP:::1',
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

function forwardedHeaders(headers, target) {
  return {
    ...headers,
    host: `${target.host}:${target.port}`,
    'x-forwarded-host': headers.host ?? 'localhost',
    'x-forwarded-proto': 'https',
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

function proxyHttpRequest(request, response, target) {
  const upstream = httpRequest({
    host: target.host,
    port: target.port,
    method: request.method,
    path: request.url,
    headers: forwardedHeaders(request.headers, target),
  });
  upstream.on('response', (upstreamResponse) => {
    response.writeHead(upstreamResponse.statusCode ?? 502, upstreamResponse.headers);
    upstreamResponse.pipe(response);
  });
  upstream.on('error', () => {
    if (!response.headersSent) {
      writeText(response, 502, 'selected IM development application is not ready\n');
    } else {
      response.destroy();
    }
  });
  request.on('aborted', () => upstream.destroy());
  request.pipe(upstream);
}

function proxyWebSocketUpgrade(request, socket, head, target) {
  const upstream = connect(target.port, target.host);
  upstream.once('connect', () => {
    const headers = forwardedHeaders(request.headers, target);
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

export function createImDevIngressServer({ certificate, privateKey, settings }) {
  const server = createServer({ cert: certificate, key: privateKey }, (request, response) => {
    const pathWithoutTrailingSlash = settings.pathPrefix.slice(0, -1);
    if (request.url === pathWithoutTrailingSlash) {
      writeText(response, 308, '', { location: settings.pathPrefix });
      return;
    }
    if (!request.url?.startsWith(settings.pathPrefix)) {
      writeText(response, 404, 'route was not found\n');
      return;
    }
    const target = selectImDevTarget(request.headers['user-agent'], settings);
    proxyHttpRequest(request, response, target);
  });
  server.on('upgrade', (request, socket, head) => {
    if (!request.url?.startsWith(settings.pathPrefix)) {
      socket.destroy();
      return;
    }
    const target = selectImDevTarget(request.headers['user-agent'], settings);
    proxyWebSocketUpgrade(request, socket, head, target);
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
    host: config.https.host,
    port: config.https.port,
    pathPrefix: config.route.pathPrefix,
    mobileUserAgentTokens: config.route.mobileUserAgentTokens,
    targets: {
      pc: config.applications.pc,
      h5: config.applications.h5,
    },
  };
  const { certificateFile, privateKeyFile } = ensureDevelopmentCertificate(
    config.https.certificate,
  );
  const server = createImDevIngressServer({
    certificate: readFileSync(certificateFile),
    privateKey: readFileSync(privateKeyFile),
    settings,
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(settings.port, settings.host, resolve);
  });
  process.stdout.write(
    `[sdkwork-web] IM dev config: ${config.configPath}\n`
      + `[sdkwork-web] IM dev ingress: https://localhost:${settings.port}${settings.pathPrefix}\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().catch((error) => {
    process.stderr.write(`[sdkwork-web] ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
