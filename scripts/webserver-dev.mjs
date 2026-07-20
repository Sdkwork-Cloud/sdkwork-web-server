#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { parseEnv } from 'node:util';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CRITICAL_SOURCE_FILES = [
  '.env.postgres.example',
  'Cargo.toml',
  'crates/sdkwork-api-web-server-standalone-gateway/Cargo.toml',
  'crates/sdkwork-api-web-server-standalone-gateway/src/main.rs',
];
const POSTGRES_ENV_PREFIX = 'SDKWORK_CLAW_DATABASE_';

function parseArgs(argv) {
  const settings = {
    database: 'postgres',
    deploymentProfile: 'standalone',
    devEnvFile: '.env.postgres',
    dryRun: false,
    environment: 'development',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--database') {
      settings.database = argv[++index];
    } else if (argument === '--deployment-profile') {
      settings.deploymentProfile = argv[++index];
    } else if (argument === '--environment') {
      settings.environment = argv[++index];
    } else if (argument === '--dev-env-file') {
      settings.devEnvFile = argv[++index];
    } else if (argument === '--dry-run') {
      settings.dryRun = true;
    } else if (argument === '--help' || argument === '-h') {
      settings.help = true;
    } else {
      throw new Error(`unsupported option: ${argument}`);
    }
  }

  if (!['postgres', 'sqlite'].includes(settings.database)) {
    throw new Error('--database must be postgres or sqlite');
  }
  if (!['standalone', 'cloud'].includes(settings.deploymentProfile)) {
    throw new Error('--deployment-profile must be standalone or cloud');
  }
  if (!['development', 'test', 'staging', 'production'].includes(settings.environment)) {
    throw new Error('--environment must be development, test, staging, or production');
  }
  if (settings.database === 'sqlite' && settings.deploymentProfile !== 'standalone') {
    throw new Error('SQLite is supported only by the standalone development profile');
  }

  return settings;
}

function printHelp() {
  console.log(`Usage: node scripts/webserver-dev.mjs [options]

Options:
  --database <postgres|sqlite>              Default: postgres
  --deployment-profile <standalone|cloud>  Default: standalone
  --environment <environment>              Default: development
  --dev-env-file <path>                    Default: .env.postgres
  --dry-run                                Print the resolved plan
  --help, -h                               Show this help`);
}

function runGit(args) {
  return spawnSync('git', args, {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    stdio: 'pipe',
    windowsHide: true,
  });
}

function ensureCriticalSources() {
  for (const relativePath of CRITICAL_SOURCE_FILES) {
    const absolutePath = path.join(REPO_ROOT, relativePath);
    if (existsSync(absolutePath)) {
      continue;
    }

    const tracked = runGit(['ls-files', '--error-unmatch', '--', relativePath]);
    const recoveryCommand = `git checkout HEAD -- ${relativePath}`;
    if (tracked.status !== 0) {
      throw new Error(
        `missing build-critical source ${relativePath}; recover it with: ${recoveryCommand}`,
      );
    }

    const recovered = runGit(['checkout', 'HEAD', '--', relativePath]);
    if (recovered.status !== 0 || !existsSync(absolutePath)) {
      throw new Error(
        `failed to recover build-critical source ${relativePath}; run: ${recoveryCommand}`,
      );
    }
    console.log(`[sdkwork-web] recovered ${relativePath} from git`);
  }
}

function resolveEnvPath(relativeOrAbsolutePath) {
  return path.isAbsolute(relativeOrAbsolutePath)
    ? relativeOrAbsolutePath
    : path.join(REPO_ROOT, relativeOrAbsolutePath);
}

function materializeDefaultPostgresEnv(envPath) {
  if (existsSync(envPath)) {
    return;
  }

  const defaultEnvPath = path.join(REPO_ROOT, '.env.postgres');
  if (path.resolve(envPath) !== defaultEnvPath) {
    throw new Error(`PostgreSQL environment file does not exist: ${envPath}`);
  }

  const examplePath = path.join(REPO_ROOT, '.env.postgres.example');
  if (!existsSync(examplePath)) {
    throw new Error(
      'missing build-critical source .env.postgres.example; run: git checkout HEAD -- .env.postgres.example',
    );
  }
  copyFileSync(examplePath, envPath);
  console.log('[sdkwork-web] created .env.postgres from .env.postgres.example');
}

function loadPostgresEnv(settings) {
  const envPath = resolveEnvPath(settings.devEnvFile);
  materializeDefaultPostgresEnv(envPath);
  const parsed = parseEnv(readFileSync(envPath, 'utf8'));
  const databaseEnv = Object.fromEntries(
    Object.entries(parsed).filter(([key]) => key.startsWith(POSTGRES_ENV_PREFIX)),
  );

  if (databaseEnv.SDKWORK_CLAW_DATABASE_ENGINE !== 'postgresql') {
    throw new Error(`${settings.devEnvFile} must set SDKWORK_CLAW_DATABASE_ENGINE=postgresql`);
  }
  return { databaseEnv, envPath };
}

function createSqliteEnv() {
  const runtimeDirectory = path.join(REPO_ROOT, '.sdkwork', 'runtime', 'webserver');
  mkdirSync(runtimeDirectory, { recursive: true });
  const databasePath = path.join(runtimeDirectory, 'webserver.sqlite').split(path.sep).join('/');
  return {
    SDKWORK_WEB_DATABASE_ENGINE: 'sqlite',
    SDKWORK_WEB_DATABASE_MAX_CONNECTIONS: '1',
    SDKWORK_WEB_DATABASE_URL: `sqlite:///${databasePath}?mode=rwc`,
  };
}

function buildRuntimeEnv(settings) {
  let databaseEnv;
  let databaseSource;
  if (settings.database === 'postgres') {
    const loaded = loadPostgresEnv(settings);
    databaseEnv = loaded.databaseEnv;
    databaseSource = path.relative(REPO_ROOT, loaded.envPath) || loaded.envPath;
  } else {
    databaseEnv = createSqliteEnv();
    databaseSource = '.sdkwork/runtime/webserver/webserver.sqlite';
  }

  return {
    databaseSource,
    env: {
      ...process.env,
      ...databaseEnv,
      SDKWORK_WEB_DATABASE_AUTO_MIGRATE: 'true',
      SDKWORK_WEB_DEPLOYMENT_PROFILE: settings.deploymentProfile,
      SDKWORK_WEB_DEV_AUTH_BYPASS: settings.environment === 'development' ? 'true' : 'false',
      SDKWORK_WEB_ENVIRONMENT: settings.environment,
      SDKWORK_WEB_RUNTIME_TARGET: 'server',
      SDKWORK_WEB_SNOWFLAKE_NODE_ID: process.env.SDKWORK_WEB_SNOWFLAKE_NODE_ID ?? '0',
    },
  };
}

async function run() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    return;
  }

  ensureCriticalSources();
  const runtime = buildRuntimeEnv(settings);
  console.log(
    `[sdkwork-web] environment=${settings.environment} deploymentProfile=${settings.deploymentProfile} runtimeTarget=server database=${settings.database}`,
  );
  console.log(`[sdkwork-web] databaseSource=${runtime.databaseSource}`);
  console.log('[sdkwork-web] managementUrl=http://127.0.0.1:3800');

  const command = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
  const args = [
    'run',
    '-p',
    'sdkwork-api-web-server-standalone-gateway',
    '--bin',
    'sdkwork-api-web-server-standalone-gateway',
  ];
  if (settings.dryRun) {
    console.log(`[sdkwork-web] command=${command} ${args.join(' ')}`);
    return;
  }

  const child = spawn(command, args, {
    cwd: REPO_ROOT,
    env: runtime.env,
    stdio: 'inherit',
    windowsHide: true,
  });

  await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0 || signal === 'SIGINT' || signal === 'SIGTERM') {
        resolve();
      } else {
        reject(new Error(`sdkwork-api-web-server-standalone-gateway exited with code ${code ?? 1}`));
      }
    });
  });
}

run().catch((error) => {
  process.stderr.write(`[sdkwork-web] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
