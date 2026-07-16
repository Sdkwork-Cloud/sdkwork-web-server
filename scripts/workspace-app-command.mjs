#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DEFAULT_CONFIG_PATH = path.join(REPO_ROOT, 'etc', 'sdkwork.workspace.json');
const EXPECTED_KIND = 'sdkwork.source-application-workspace';

function parseArgs(argv) {
  const settings = {
    configPath: DEFAULT_CONFIG_PATH,
    dryRun: false,
    platform: undefined,
  };
  const positionals = [];

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--config') {
      settings.configPath = path.resolve(argv[++index] ?? '');
    } else if (argument === '--platform') {
      settings.platform = argv[++index];
    } else if (argument === '--dry-run') {
      settings.dryRun = true;
    } else if (argument === '--help' || argument === '-h') {
      settings.help = true;
    } else if (argument.startsWith('--')) {
      throw new Error(`unsupported option: ${argument}`);
    } else {
      positionals.push(argument);
    }
  }

  settings.operation = positionals[0];
  settings.applicationId = positionals[1];
  return settings;
}

function printHelp() {
  console.log(`Usage:
  node scripts/workspace-app-command.mjs build <application-id> [--dry-run]
  node scripts/workspace-app-command.mjs package <application-id> --platform <ios|android> [--dry-run]

Configuration defaults to etc/sdkwork.workspace.json.`);
}

function pathIsInside(parentPath, candidatePath) {
  const relative = path.relative(parentPath, candidatePath);
  return relative === '' || (!relative.startsWith('..') && !path.isAbsolute(relative));
}

function readWorkspaceConfig(configPath) {
  if (!existsSync(configPath)) {
    throw new Error(`workspace configuration does not exist: ${configPath}`);
  }
  const config = JSON.parse(readFileSync(configPath, 'utf8'));
  if (config.schemaVersion !== 1 || config.kind !== EXPECTED_KIND) {
    throw new Error(`unsupported workspace configuration: ${configPath}`);
  }
  if (!config.applications || typeof config.applications !== 'object') {
    throw new Error('workspace configuration must define applications');
  }
  return config;
}

function runGit(repositoryRoot, args) {
  return spawnSync('git', ['-C', repositoryRoot, ...args], {
    encoding: 'utf8',
    stdio: 'pipe',
    windowsHide: true,
  });
}

function ensureCriticalFiles(applicationRoot, repositoryRoot, criticalFiles) {
  for (const relativePath of criticalFiles ?? []) {
    const absolutePath = path.resolve(applicationRoot, relativePath);
    if (!pathIsInside(applicationRoot, absolutePath)) {
      throw new Error(`critical file escapes application root: ${relativePath}`);
    }
    if (existsSync(absolutePath)) {
      continue;
    }

    const repositoryRelativePath = path.relative(repositoryRoot, absolutePath).split(path.sep).join('/');
    const recoveryCommand = `git -C ${repositoryRoot} checkout HEAD -- ${repositoryRelativePath}`;
    const tracked = runGit(repositoryRoot, [
      'ls-files',
      '--error-unmatch',
      '--',
      repositoryRelativePath,
    ]);
    if (tracked.status !== 0) {
      throw new Error(`missing build-critical source ${absolutePath}; recover it with: ${recoveryCommand}`);
    }
    const recovered = runGit(repositoryRoot, ['checkout', 'HEAD', '--', repositoryRelativePath]);
    if (recovered.status !== 0 || !existsSync(absolutePath)) {
      throw new Error(`failed to recover build-critical source ${absolutePath}; run: ${recoveryCommand}`);
    }
    console.log(`[sdkwork-workspace] recovered ${repositoryRelativePath} from git`);
  }
}

function resolvePlan(settings) {
  if (!settings.operation || !settings.applicationId) {
    throw new Error('operation and application id are required; run with --help');
  }
  if (!['build', 'package'].includes(settings.operation)) {
    throw new Error('operation must be build or package');
  }
  if (settings.operation === 'package' && !['ios', 'android'].includes(settings.platform)) {
    throw new Error('package requires --platform ios or --platform android');
  }

  const config = readWorkspaceConfig(settings.configPath);
  const configDirectory = path.dirname(settings.configPath);
  const workspaceRoot = path.resolve(configDirectory, config.workspaceRoot);
  const application = config.applications[settings.applicationId];
  if (!application) {
    throw new Error(
      `unknown application ${settings.applicationId}; available: ${Object.keys(config.applications).join(', ')}`,
    );
  }
  const repository = config.repositories?.[application.repository];
  if (!repository) {
    throw new Error(`application ${settings.applicationId} references unknown repository ${application.repository}`);
  }

  const repositoryRoot = path.resolve(workspaceRoot, repository.root);
  const applicationRoot = path.resolve(workspaceRoot, application.root);
  if (!pathIsInside(workspaceRoot, repositoryRoot) || !pathIsInside(repositoryRoot, applicationRoot)) {
    throw new Error(`application path escapes configured workspace: ${application.root}`);
  }
  if (!existsSync(repositoryRoot) || !existsSync(applicationRoot)) {
    throw new Error(`configured application root does not exist: ${applicationRoot}`);
  }

  const actionId = settings.operation === 'build' ? 'build' : `package:${settings.platform}`;
  const action = application.actions?.[actionId];
  if (!action) {
    throw new Error(`${settings.applicationId} does not support ${actionId}`);
  }
  if (!Array.isArray(action.steps) || action.steps.length === 0) {
    throw new Error(`${settings.applicationId} action ${actionId} has no steps`);
  }
  if (!settings.dryRun && action.supportedHosts && !action.supportedHosts.includes(process.platform)) {
    throw new Error(
      `${settings.applicationId} ${actionId} requires ${action.supportedHosts.join(' or ')}; current host is ${process.platform}`,
    );
  }

  const steps = action.steps.map((step) => {
    const cwd = path.resolve(applicationRoot, step.cwd ?? '.');
    if (!pathIsInside(applicationRoot, cwd)) {
      throw new Error(`action working directory escapes application root: ${step.cwd}`);
    }
    return { command: step.command, args: step.args ?? [], cwd };
  });
  return {
    action,
    actionId,
    application,
    applicationRoot,
    repositoryRoot,
    steps,
  };
}

function executableName(command) {
  if (process.platform !== 'win32') {
    return command;
  }
  return command === 'pnpm' ? 'pnpm.cmd' : command === 'flutter' ? 'flutter.bat' : command;
}

async function runStep(step) {
  const command = executableName(step.command);
  console.log(`[sdkwork-workspace] run cwd=${step.cwd} command=${command} ${step.args.join(' ')}`);
  const child = spawn(command, step.args, {
    cwd: step.cwd,
    env: process.env,
    stdio: 'inherit',
    windowsHide: true,
  });
  await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${step.command} exited with ${signal ? `signal ${signal}` : `code ${code ?? 1}`}`));
      }
    });
  });
}

async function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    return;
  }
  const plan = resolvePlan(settings);
  ensureCriticalFiles(plan.applicationRoot, plan.repositoryRoot, plan.application.criticalFiles);
  console.log(
    `[sdkwork-workspace] application=${settings.applicationId} canonicalAppKey=${plan.application.canonicalAppKey} action=${plan.actionId} runtimeTarget=${plan.action.runtimeTarget}`,
  );
  console.log(`[sdkwork-workspace] root=${plan.applicationRoot}`);
  for (const step of plan.steps) {
    if (settings.dryRun) {
      console.log(`[sdkwork-workspace] plan cwd=${step.cwd} command=${step.command} ${step.args.join(' ')}`);
    } else {
      await runStep(step);
    }
  }
}

main().catch((error) => {
  process.stderr.write(`[sdkwork-workspace] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
