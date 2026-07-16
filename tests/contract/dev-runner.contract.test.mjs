import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('root dev command starts the device-aware IM development ingress', () => {
  const packageJson = JSON.parse(readFileSync(path.join(REPO_ROOT, 'package.json'), 'utf8'));

  assert.equal(packageJson.scripts.dev, 'pnpm dev:browser');
  assert.equal(packageJson.scripts['dev:browser'], 'pnpm dev:browser:postgres:standalone');
  assert.match(packageJson.scripts['dev:browser:postgres:standalone'], /webserver-im-dev\.mjs/u);
  assert.match(
    packageJson.scripts['dev:browser:postgres:standalone'],
    /--config etc\/sdkwork\.webserver\.im-dev\.json/u,
  );
  assert.equal(packageJson.scripts['dev:server'], 'pnpm dev:server:postgres:standalone');
  assert.match(packageJson.scripts['dev:server:postgres:standalone'], /--database postgres/u);
  assert.match(packageJson.scripts['dev:server:postgres:standalone'], /--environment development/u);
  assert.match(
    packageJson.scripts['dev:server:postgres:standalone'],
    /--deployment-profile standalone/u,
  );
});

test('IM dev plan mounts both applications at the shared path', () => {
  const result = spawnSync(process.execPath, ['scripts/webserver-im-dev.mjs', '--dry-run'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    windowsHide: true,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /sdkwork-im-pc/u);
  assert.match(result.stdout, /sdkwork-im-h5/u);
  assert.match(result.stdout, /etc[\\/]sdkwork\.webserver\.im-dev\.json/u);
  assert.match(result.stdout, /--base \/sdkwork-im\//u);
  assert.doesNotMatch(result.stdout, / dev -- --host/u);
  assert.match(result.stdout, /dev-im-ingress\.mjs/u);
});

test('SQLite development remains an explicit server variant', () => {
  const result = spawnSync(
    process.execPath,
    [
      'scripts/webserver-dev.mjs',
      '--database',
      'sqlite',
      '--deployment-profile',
      'standalone',
      '--environment',
      'development',
      '--dry-run',
    ],
    {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      env: { ...process.env, SDKWORK_WEB_SNOWFLAKE_NODE_ID: '7' },
      windowsHide: true,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /database=sqlite/u);
  assert.match(result.stdout, /databaseSource=\.sdkwork\/runtime\/webserver\/webserver\.sqlite/u);
  assert.match(result.stdout, /managementUrl=http:\/\/127\.0\.0\.1:3800/u);
  assert.doesNotMatch(result.stdout, /SDKWORK_CLAW_DATABASE_PASSWORD/u);
});
