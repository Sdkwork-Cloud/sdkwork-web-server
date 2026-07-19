#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DEFAULT_INDEX = path.join(REPO_ROOT, 'etc', 'sdkwork.deployment.config.json');

function parseArgs(argv) {
  const settings = {
    deploymentProfile: 'cloud',
    environment: 'development',
    configPath: DEFAULT_INDEX,
    dryRun: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--deployment-profile') {
      settings.deploymentProfile = argv[++index];
    } else if (argument === '--environment') {
      settings.environment = argv[++index];
    } else if (argument === '--config') {
      settings.configPath = path.resolve(argv[++index] ?? '');
    } else if (argument === '--dry-run') {
      settings.dryRun = true;
    } else if (argument === '--help' || argument === '-h') {
      settings.help = true;
    } else {
      throw new Error(`unsupported option: ${argument}`);
    }
  }
  return settings;
}

function readJson(filePath, label) {
  if (!existsSync(filePath)) {
    throw new Error(`${label} does not exist: ${filePath}`);
  }
  try {
    return JSON.parse(readFileSync(filePath, 'utf8'));
  } catch (error) {
    throw new Error(`invalid ${label}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function assertExactKeys(value, keys, label) {
  const actual = Object.keys(value ?? {}).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw new Error(`${label} must contain exactly: ${expected.join(', ')}`);
  }
}

function validateRemoteOrigin(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error('cloud.development backendApiBaseUrl must be a valid URL');
  }
  const hostname = url.hostname.toLowerCase();
  if (
    url.protocol !== 'https:' ||
    url.username ||
    url.password ||
    url.pathname !== '/' ||
    url.search ||
    url.hash ||
    hostname === 'localhost' ||
    hostname === '0.0.0.0' ||
    hostname === '::1' ||
    hostname.startsWith('127.')
  ) {
    throw new Error(
      'cloud.development backendApiBaseUrl must be a remote HTTPS origin without credentials, path, query, or fragment',
    );
  }
  return url.origin;
}

function resolveProfile(settings) {
  if (settings.deploymentProfile !== 'cloud' || settings.environment !== 'development') {
    throw new Error('webserver-cloud-dev supports only cloud.development');
  }
  const index = readJson(settings.configPath, 'deployment index');
  assertExactKeys(index, ['schemaVersion', 'kind', 'application', 'defaultProfile', 'profiles'], 'deployment index');
  if (index.schemaVersion !== 1 || index.kind !== 'sdkwork.deployment-index') {
    throw new Error('unsupported deployment index contract');
  }
  const profileId = `${settings.deploymentProfile}.${settings.environment}`;
  const reference = index.profiles?.[profileId];
  assertExactKeys(reference, ['config'], `deployment profile ${profileId}`);
  const profilePath = path.resolve(path.dirname(settings.configPath), reference.config);
  const etcRoot = path.resolve(REPO_ROOT, 'etc');
  const relative = path.relative(etcRoot, profilePath);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`deployment profile ${profileId} escapes etc/`);
  }
  const profile = readJson(profilePath, `deployment profile ${profileId}`);
  assertExactKeys(
    profile,
    ['schemaVersion', 'kind', 'environment', 'deploymentProfile', 'runtimeTarget', 'apiSurfaces'],
    profileId,
  );
  assertExactKeys(profile.apiSurfaces, ['backendApiBaseUrl'], `${profileId}.apiSurfaces`);
  if (
    profile.schemaVersion !== 1 ||
    profile.kind !== 'sdkwork.web-node-daemon-profile' ||
    profile.environment !== 'development' ||
    profile.deploymentProfile !== 'cloud' ||
    profile.runtimeTarget !== 'server'
  ) {
    throw new Error(`invalid ${profileId} identity`);
  }
  return {
    profileId,
    backendApiBaseUrl: validateRemoteOrigin(profile.apiSurfaces.backendApiBaseUrl),
  };
}

async function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    console.log('Usage: node scripts/webserver-cloud-dev.mjs --deployment-profile cloud --environment development [--dry-run]');
    return;
  }
  const profile = resolveProfile(settings);
  const tokenConfigured = Boolean(
    process.env.SDKWORK_WEB_NODE_TOKEN || process.env.SDKWORK_WEB_AGENT_TOKEN,
  );
  console.log(
    `[sdkwork-web] profile=${profile.profileId} runtimeTarget=server localProcess=web-node-daemon`,
  );
  console.log(`[sdkwork-web] backendApiBaseUrl=${profile.backendApiBaseUrl}`);
  console.log(`[sdkwork-web] nodeTokenConfigured=${tokenConfigured}`);
  const command = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
  const args = ['run', '-p', 'sdkwork-web-agent', '--bin', 'sdkwork-web-node-daemon'];
  if (settings.dryRun) {
    console.log(`[sdkwork-web] command=${command} ${args.join(' ')}`);
    return;
  }
  if (!tokenConfigured) {
    throw new Error(
      'SDKWORK_WEB_NODE_TOKEN is required locally for cloud.development; no token is read from tracked config',
    );
  }
  const child = spawn(command, args, {
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      SDKWORK_WEB_CONTROL_PLANE_URL: profile.backendApiBaseUrl,
      SDKWORK_WEB_DEPLOYMENT_PROFILE: 'cloud',
      SDKWORK_WEB_ENVIRONMENT: 'development',
      SDKWORK_WEB_RUNTIME_TARGET: 'server',
    },
    stdio: 'inherit',
    windowsHide: true,
  });
  await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0 || signal === 'SIGINT' || signal === 'SIGTERM') {
        resolve();
      } else {
        reject(new Error(`Web Node Daemon exited with code ${code ?? 1}`));
      }
    });
  });
}

main().catch((error) => {
  process.stderr.write(`[sdkwork-web] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
