#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import http from 'node:http';
import https from 'node:https';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { extract as extractTar } from 'tar';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const OUTPUT_ROOT = path.join(REPO_ROOT, 'dist', 'release');
const MAX_PROCESS_OUTPUT_BYTES = 256 * 1024;
const MAX_RESPONSE_BYTES = 64 * 1024;
const COMMAND_TIMEOUT_MS = 30 * 1000;
const START_TIMEOUT_MS = 15 * 1000;
const STOP_TIMEOUT_MS = 10 * 1000;
const SUPPORTED_ARCHITECTURES = new Set(['x64', 'arm64']);
const EXPECTED_BINARIES = [
  'sdkwork-web-standalone-gateway',
  'sdkwork-web-node-daemon',
  'sdkwork-web-agent',
  'sdkwork-webserver-certificate-worker',
];

function parseArgs(argv) {
  const settings = {
    deploymentProfile: process.env.SDKWORK_DEPLOYMENT_PROFILE,
    architecture: process.env.SDKWORK_PACKAGE_ARCHITECTURE,
    version: undefined,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--deployment-profile') {
      settings.deploymentProfile = argv[++index];
    } else if (argument === '--architecture') {
      settings.architecture = argv[++index];
    } else if (argument === '--version') {
      settings.version = argv[++index];
    } else if (argument === '--help' || argument === '-h') {
      settings.help = true;
    } else {
      throw new Error(`unsupported option: ${argument}`);
    }
  }
  return settings;
}

function resolveArtifact(settings) {
  if (!['standalone', 'cloud'].includes(settings.deploymentProfile)) {
    throw new Error('--deployment-profile must be standalone or cloud');
  }
  const manifest = JSON.parse(
    readFileSync(path.join(REPO_ROOT, 'sdkwork.app.config.json'), 'utf8'),
  );
  const packageVersion = process.env.SDKWORK_PACKAGE_VERSION?.trim();
  const compatibilityVersion = process.env.SDKWORK_RELEASE_VERSION?.trim();
  if (packageVersion && compatibilityVersion && packageVersion !== compatibilityVersion) {
    throw new Error('SDKWORK_PACKAGE_VERSION conflicts with SDKWORK_RELEASE_VERSION');
  }
  const version =
    settings.version || packageVersion || compatibilityVersion || manifest.release?.defaultVersion;
  if (!/^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/u.test(version ?? '')) {
    throw new Error('release version must be an explicit semantic version');
  }
  const architecture = settings.architecture?.trim() || process.arch;
  if (!SUPPORTED_ARCHITECTURES.has(architecture)) {
    throw new Error('release architecture must be x64 or arm64');
  }
  const artifactBase = `sdkwork-web-linux-${architecture}-${settings.deploymentProfile}-server-${version}`;
  return {
    version,
    architecture,
    artifactBase,
    archive: path.join(OUTPUT_ROOT, `${artifactBase}.tar.gz`),
  };
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? REPO_ROOT,
    encoding: 'utf8',
    env: options.env ?? process.env,
    stdio: 'pipe',
    timeout: options.timeoutMs ?? COMMAND_TIMEOUT_MS,
    maxBuffer: MAX_PROCESS_OUTPUT_BYTES,
    killSignal: 'SIGKILL',
    windowsHide: true,
  });
  if (result.error || result.status !== 0) {
    const detail =
      result.error?.message || result.stderr?.trim() || result.stdout?.trim() || `exit ${result.status}`;
    throw new Error(`${command} ${args.join(' ')} failed: ${detail}`);
  }
  return result;
}

function captureBounded(stream) {
  const chunks = [];
  let retainedBytes = 0;
  stream.on('data', (chunk) => {
    if (retainedBytes >= MAX_PROCESS_OUTPUT_BYTES) {
      return;
    }
    const retained = Buffer.from(chunk).subarray(0, MAX_PROCESS_OUTPUT_BYTES - retainedBytes);
    chunks.push(retained);
    retainedBytes += retained.length;
  });
  return () => Buffer.concat(chunks, retainedBytes).toString('utf8');
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : undefined;
      server.close((error) => {
        if (error) {
          reject(error);
        } else if (!port) {
          reject(new Error('ephemeral port allocation returned no port'));
        } else {
          resolve(port);
        }
      });
    });
  });
}

function requestHealth(protocol, port) {
  return new Promise((resolve, reject) => {
    const transport = protocol === 'https' ? https : http;
    const request = transport.request(
      {
        host: '127.0.0.1',
        port,
        path: '/healthz',
        method: 'GET',
        headers: { host: 'localhost' },
        rejectUnauthorized: protocol === 'https' ? false : undefined,
        servername: protocol === 'https' ? 'localhost' : undefined,
      },
      (response) => {
        const chunks = [];
        let bytes = 0;
        response.on('data', (chunk) => {
          bytes += chunk.length;
          if (bytes > MAX_RESPONSE_BYTES) {
            request.destroy(new Error(`smoke response exceeds ${MAX_RESPONSE_BYTES} bytes`));
            return;
          }
          chunks.push(Buffer.from(chunk));
        });
        response.once('error', reject);
        response.once('end', () => {
          resolve({
            statusCode: response.statusCode,
            body: Buffer.concat(chunks, bytes).toString('utf8'),
          });
        });
      },
    );
    request.setTimeout(2_000, () => request.destroy(new Error('smoke request timed out')));
    request.once('error', reject);
    request.end();
  });
}

async function waitForHealth(protocol, port, child, readOutput) {
  const deadline = Date.now() + START_TIMEOUT_MS;
  let lastError;
  while (Date.now() < deadline) {
    if (child.exitCode !== null || child.signalCode !== null) {
      throw new Error(`packaged gateway exited before readiness: ${readOutput()}`);
    }
    try {
      const response = await requestHealth(protocol, port);
      if (response.statusCode === 200 && response.body === 'release-smoke\n') {
        return;
      }
      lastError = new Error(
        `${protocol} health returned status=${response.statusCode} body=${JSON.stringify(response.body)}`,
      );
    } catch (error) {
      lastError = error;
    }
    await delay(100);
  }
  throw new Error(`${protocol} health did not become ready: ${lastError?.message ?? 'unknown'}`);
}

function waitForExit(child, timeoutMs) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return Promise.resolve({ code: child.exitCode, signal: child.signalCode });
  }
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('packaged gateway stop timed out')), timeoutMs);
    child.once('error', (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.once('exit', (code, signal) => {
      clearTimeout(timeout);
      resolve({ code, signal });
    });
  });
}

function buildSmokeConfig(example, httpPort, httpsPort, certificateFile, privateKeyFile) {
  return {
    ...example,
    appKey: 'sdkwork-release-smoke',
    limits: {
      ...example.limits,
      maxConnections: 128,
      maxConcurrentRequests: 64,
      maxConcurrentHealthChecks: 4,
      drainTimeoutMs: 1_000,
      maxConnectionAgeMs: 60_000,
    },
    listeners: [
      {
        id: 'smoke-http',
        bind: '127.0.0.1',
        port: httpPort,
        protocols: ['http1'],
        defaultVirtualHostRef: 'smoke-host',
      },
      {
        id: 'smoke-https',
        bind: '127.0.0.1',
        port: httpsPort,
        protocols: ['http1', 'http2'],
        tlsPolicyRef: 'smoke-tls',
        defaultVirtualHostRef: 'smoke-host',
      },
    ],
    certificates: [
      {
        id: 'smoke-certificate',
        serverNames: ['localhost'],
        source: {
          type: 'protected-file',
          certificateFile,
          privateKeyFile,
        },
      },
    ],
    tlsPolicies: [
      {
        id: 'smoke-tls',
        certificateRef: 'smoke-certificate',
        minimumVersion: 'tls1.2',
        maximumVersion: 'tls1.3',
        alpn: ['h2', 'http/1.1'],
      },
    ],
    resolvers: [],
    resources: [
      {
        id: 'smoke-response',
        type: 'respond',
        status: 200,
        contentType: 'text/plain; charset=utf-8',
        body: 'release-smoke\n',
      },
    ],
    upstreams: [],
    virtualHosts: [
      {
        id: 'smoke-host',
        listenerRefs: ['smoke-http', 'smoke-https'],
        serverNames: ['localhost'],
        routes: [
          {
            id: 'health',
            match: { pathType: 'exact', path: '/healthz', methods: ['GET', 'HEAD'] },
            resourceRef: 'smoke-response',
          },
        ],
      },
    ],
    observability: { accessLog: false },
    deployment: {
      drainTimeoutMs: 1_000,
      reload: { mode: 'disabled' },
    },
    metadata: { owner: 'sdkwork-release-smoke' },
  };
}

async function smoke(settings) {
  const resolved = resolveArtifact(settings);
  if (process.platform !== 'linux' || process.arch !== resolved.architecture) {
    throw new Error(
      `Linux ${resolved.architecture} release smoke must run on a linux-${resolved.architecture} host`,
    );
  }
  run(process.execPath, [
    'scripts/webserver-release.mjs',
    'validate',
    '--deployment-profile',
    settings.deploymentProfile,
    '--architecture',
    resolved.architecture,
    '--version',
    resolved.version,
  ]);

  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-web-release-smoke-'));
  let child;
  try {
    await extractTar({
      file: resolved.archive,
      cwd: temporaryRoot,
      strict: true,
      preservePaths: false,
    });
    const packageRoot = path.join(temporaryRoot, 'sdkwork-web');
    const binRoot = path.join(packageRoot, 'bin');
    for (const binary of EXPECTED_BINARIES) {
      const metadata = statSync(path.join(binRoot, binary));
      if (!metadata.isFile() || (metadata.mode & 0o111) === 0) {
        throw new Error(`packaged binary ${binary} is not an executable regular file`);
      }
    }

    const gateway = path.join(binRoot, 'sdkwork-web-standalone-gateway');
    const packagedExample = path.join(packageRoot, 'etc', 'examples', 'sdkwork.webserver.config.json');
    run(gateway, ['--help'], { cwd: packageRoot });
    run(gateway, ['validate', packagedExample], { cwd: packageRoot });

    const certificateFile = path.join(temporaryRoot, 'smoke-cert.pem');
    const privateKeyFile = path.join(temporaryRoot, 'smoke-key.pem');
    run('openssl', [
      'req',
      '-x509',
      '-newkey',
      'rsa:2048',
      '-sha256',
      '-nodes',
      '-days',
      '1',
      '-subj',
      '/CN=localhost',
      '-addext',
      'subjectAltName=DNS:localhost',
      '-keyout',
      privateKeyFile,
      '-out',
      certificateFile,
    ]);

    const httpPort = await reservePort();
    let httpsPort = await reservePort();
    while (httpsPort === httpPort) {
      httpsPort = await reservePort();
    }
    const example = JSON.parse(readFileSync(packagedExample, 'utf8'));
    const smokeConfig = buildSmokeConfig(
      example,
      httpPort,
      httpsPort,
      certificateFile,
      privateKeyFile,
    );
    const smokeConfigPath = path.join(temporaryRoot, 'sdkwork.webserver.release-smoke.json');
    writeFileSync(smokeConfigPath, `${JSON.stringify(smokeConfig, null, 2)}\n`, {
      encoding: 'utf8',
      flag: 'wx',
      mode: 0o600,
    });
    run(gateway, ['validate', smokeConfigPath], { cwd: packageRoot });

    child = spawn(gateway, ['data-plane', smokeConfigPath], {
      cwd: packageRoot,
      env: {
        ...process.env,
        RUST_LOG: 'info',
        SDKWORK_WEB_ENVIRONMENT: 'test',
        SDKWORK_WEB_DEPLOYMENT_PROFILE: settings.deploymentProfile,
        SDKWORK_WEB_RUNTIME_TARGET: 'server',
      },
      stdio: ['ignore', 'pipe', 'pipe'],
      windowsHide: true,
    });
    const readStdout = captureBounded(child.stdout);
    const readStderr = captureBounded(child.stderr);
    const readOutput = () => `${readStdout()}\n${readStderr()}`.trim();

    await waitForHealth('http', httpPort, child, readOutput);
    await waitForHealth('https', httpsPort, child, readOutput);
    child.kill('SIGTERM');
    let exit;
    try {
      exit = await waitForExit(child, STOP_TIMEOUT_MS);
    } catch (error) {
      child.kill('SIGKILL');
      throw error;
    }
    if (exit.code !== 0 || exit.signal !== null) {
      throw new Error(
        `packaged gateway exited unexpectedly code=${exit.code} signal=${exit.signal}: ${readOutput()}`,
      );
    }
    child = undefined;
    console.log(
      `[sdkwork-web-release-smoke] passed artifact=${resolved.artifactBase}.tar.gz http=${httpPort} https=${httpsPort}`,
    );
  } finally {
    if (child && child.exitCode === null && child.signalCode === null) {
      child.kill('SIGKILL');
      await waitForExit(child, 5_000).catch(() => {});
    }
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
}

async function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    console.log(
      'Usage: node scripts/webserver-release-smoke.mjs --deployment-profile <standalone|cloud> [--architecture <x64|arm64>] [--version <semver>]',
    );
    return;
  }
  await smoke(settings);
}

main().catch((error) => {
  process.stderr.write(
    `[sdkwork-web-release-smoke] ${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
});
