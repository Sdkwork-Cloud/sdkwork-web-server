#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  loadImDevConfig,
  resolveImDevConfigPath,
} from './lib/im-dev-config.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const isWindows = process.platform === 'win32';
const pnpm = isWindows ? 'pnpm.cmd' : 'pnpm';

export function createImDevPlan(config, env = process.env) {
  const baseArgs = ['--base', config.route.pathPrefix, '--strictPort'];
  return {
    config,
    env,
    processes: [
      {
        label: 'sdkwork-im-pc',
        command: pnpm,
        args: [
          '--dir', config.applications.pc.root, 'dev',
          '--host', config.applications.pc.host,
          '--port', String(config.applications.pc.port),
          ...baseArgs,
        ],
        cwd: config.applications.pc.root,
      },
      {
        label: 'sdkwork-im-h5',
        command: pnpm,
        args: [
          '--dir', config.applications.h5.root, 'dev',
          '--host', config.applications.h5.host,
          '--port', String(config.applications.h5.port),
          ...baseArgs,
        ],
        cwd: config.applications.h5.root,
      },
      {
        label: 'sdkwork-im-dev-ingress',
        command: process.execPath,
        args: [
          path.join(REPO_ROOT, 'scripts', 'dev-im-ingress.mjs'),
          '--config',
          config.configPath,
        ],
        cwd: REPO_ROOT,
      },
    ],
  };
}

function terminateProcessTree(child) {
  if (!child?.pid) {
    return;
  }
  if (isWindows) {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
  } else {
    child.kill('SIGTERM');
  }
}

async function run() {
  const configArgumentIndex = process.argv.indexOf('--config');
  const configPath = resolveImDevConfigPath(
    configArgumentIndex >= 0 ? process.argv[configArgumentIndex + 1] : undefined,
  );
  const config = loadImDevConfig(configPath);
  const plan = createImDevPlan(config);
  if (process.argv.includes('--dry-run')) {
    process.stdout.write(`[sdkwork-im-dev] config ${config.configPath}\n`);
    for (const entry of plan.processes) {
      process.stdout.write(`[${entry.label}] ${entry.command} ${entry.args.join(' ')}\n`);
    }
    return;
  }

  const children = [];
  let shuttingDown = false;
  const shutdown = (failedChild) => {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    for (const child of children) {
      if (child !== failedChild && child.exitCode === null && child.signalCode === null) {
        terminateProcessTree(child);
      }
    }
  };

  for (const entry of plan.processes) {
    const child = spawn(entry.command, entry.args, {
      cwd: entry.cwd,
      env: plan.env,
      shell: isWindows && entry.command === pnpm,
      stdio: 'inherit',
      windowsHide: true,
    });
    children.push(child);
    child.on('error', (error) => {
      process.stderr.write(`[${entry.label}] ${error.message}\n`);
      process.exitCode = 1;
      shutdown(child);
    });
    child.on('exit', (code, signal) => {
      if (!shuttingDown) {
        if (code !== 0 && signal !== 'SIGINT' && signal !== 'SIGTERM') {
          process.stderr.write(`[${entry.label}] exited with code ${code ?? 1}\n`);
          process.exitCode = code ?? 1;
        }
        shutdown(child);
      }
    });
  }

  process.once('SIGINT', () => shutdown());
  process.once('SIGTERM', () => shutdown());
  await Promise.all(children.map((child) => new Promise((resolve) => child.once('exit', resolve))));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  run().catch((error) => {
    process.stderr.write(`[sdkwork-web] ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
