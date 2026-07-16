import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const RUNNER = path.join(REPO_ROOT, 'scripts', 'workspace-app-command.mjs');

function run(args) {
  return spawnSync(process.execPath, [RUNNER, ...args, '--dry-run'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    windowsHide: true,
  });
}

test('root exposes standard application build and release package commands', () => {
  const packageJson = JSON.parse(readFileSync(path.join(REPO_ROOT, 'package.json'), 'utf8'));
  assert.equal(packageJson.scripts['build:app'], 'node scripts/workspace-app-command.mjs build');
  assert.match(packageJson.scripts['release:package:ios'], /--platform ios/u);
  assert.match(packageJson.scripts['release:package:android'], /--platform android/u);
  assert.equal(packageJson.scripts['build-app'], undefined);
});

for (const applicationId of ['sdkwork-im-pc', 'sdkwork-im-h5', 'sdkwork-im-flutter']) {
  test(`resolves ${applicationId} build from etc workspace configuration`, () => {
    const result = run(['build', applicationId]);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, new RegExp(`application=${applicationId}`, 'u'));
    assert.match(result.stdout, /action=build/u);
  });
}

for (const applicationId of ['sdkwork-im-h5', 'sdkwork-im-flutter']) {
  for (const platform of ['ios', 'android']) {
    test(`resolves ${applicationId} ${platform} package`, () => {
      const result = run(['package', applicationId, '--platform', platform]);
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, new RegExp(`action=package:${platform}`, 'u'));
    });
  }
}

test('rejects unsupported application package targets', () => {
  const result = run(['package', 'sdkwork-im-pc', '--platform', 'android']);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not support package:android/u);
});
