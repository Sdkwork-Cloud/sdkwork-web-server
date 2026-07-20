import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function readJson(relativePath) {
  return JSON.parse(readFileSync(path.join(REPO_ROOT, relativePath), 'utf8'));
}

function runNode(args, env = {}) {
  const inherited = { ...process.env };
  delete inherited.SDKWORK_WEB_NODE_TOKEN;
  delete inherited.SDKWORK_WEB_AGENT_TOKEN;
  delete inherited.SDKWORK_PACKAGE_VERSION;
  delete inherited.SDKWORK_RELEASE_VERSION;
  delete inherited.SDKWORK_PACKAGE_ARCHITECTURE;
  return spawnSync(process.execPath, args, {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    env: { ...inherited, ...env },
    windowsHide: true,
  });
}

test('root development commands select explicit standalone and cloud development profiles', () => {
  const packageJson = readJson('package.json');
  assert.equal(packageJson.scripts.dev, 'pnpm dev:standalone');
  assert.equal(packageJson.scripts['dev:standalone'], 'pnpm dev:browser:postgres:standalone');
  assert.match(packageJson.scripts['dev:browser:postgres:standalone'], /--deployment-profile standalone/u);
  assert.match(packageJson.scripts['dev:browser:postgres:standalone'], /--environment development/u);
  assert.equal(
    packageJson.scripts['dev:cloud'],
    'node scripts/webserver-cloud-dev.mjs --deployment-profile cloud --environment development',
  );

  const index = readJson('etc/sdkwork.deployment.config.json');
  assert.equal(index.defaultProfile, 'standalone.development');
  assert.equal(index.profiles['cloud.development'].config, 'node-daemon/cloud.development.json');
});

test('cloud development uses one remote HTTPS control plane and starts only the Node Daemon', () => {
  const profile = readJson('etc/node-daemon/cloud.development.json');
  assert.deepEqual(Object.keys(profile).sort(), [
    'apiSurfaces',
    'deploymentProfile',
    'environment',
    'kind',
    'runtimeTarget',
    'schemaVersion',
  ]);
  assert.equal(profile.deploymentProfile, 'cloud');
  assert.equal(profile.environment, 'development');
  assert.equal(profile.runtimeTarget, 'server');
  const controlPlane = new URL(profile.apiSurfaces.backendApiBaseUrl);
  assert.equal(controlPlane.protocol, 'https:');
  assert.notEqual(controlPlane.hostname, 'localhost');
  assert.doesNotMatch(JSON.stringify(profile), /token|credential|secret/iu);

  const result = runNode([
    'scripts/webserver-cloud-dev.mjs',
    '--deployment-profile',
    'cloud',
    '--environment',
    'development',
    '--dry-run',
  ]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /profile=cloud\.development runtimeTarget=server localProcess=web-node-daemon/u);
  assert.match(result.stdout, /backendApiBaseUrl=https:\/\/api-dev\.sdkwork\.com/u);
  assert.match(result.stdout, /nodeTokenConfigured=false/u);
  assert.match(result.stdout, /run -p sdkwork-web-agent --bin sdkwork-web-node-daemon/u);
  assert.doesNotMatch(result.stdout, /webserver-dev|standalone-gateway|database|api-server/iu);

  const source = readFileSync(path.join(REPO_ROOT, 'scripts/webserver-cloud-dev.mjs'), 'utf8');
  assert.equal(source.match(/\bspawn\(/gu)?.length, 1);
  assert.doesNotMatch(source, /webserver-dev\.mjs|sdkwork-api-web-server-standalone-gateway|database host/iu);
  const envExample = readFileSync(path.join(REPO_ROOT, 'etc/agent/development.env.example'), 'utf8');
  assert.match(envExample, /^SDKWORK_WEB_NODE_TOKEN=$/mu);
});

test('release dry-runs produce distinct profile and workflow-version-bound artifact names', () => {
  const packageJson = readJson('package.json');
  assert.equal(
    packageJson.scripts['release:package:standalone'],
    'node scripts/webserver-release.mjs package --deployment-profile standalone',
  );
  assert.equal(
    packageJson.scripts['release:package:cloud'],
    'node scripts/webserver-release.mjs package --deployment-profile cloud',
  );

  for (const architecture of ['x64', 'arm64']) {
    for (const deploymentProfile of ['standalone', 'cloud']) {
      const result = runNode(
        ['scripts/webserver-release.mjs', 'package', '--deployment-profile', deploymentProfile, '--dry-run'],
        { SDKWORK_PACKAGE_VERSION: '9.8.7', SDKWORK_PACKAGE_ARCHITECTURE: architecture },
      );
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, new RegExp(`deploymentProfile=${deploymentProfile}`, 'u'));
      assert.match(result.stdout, new RegExp(`architecture=${architecture}`, 'u'));
      assert.match(
        result.stdout,
        new RegExp(`artifact=sdkwork-web-linux-${architecture}-${deploymentProfile}-server-9\\.8\\.7\\.tar\\.gz`, 'u'),
      );
    }
  }

  const conflict = runNode(
    ['scripts/webserver-release.mjs', 'package', '--deployment-profile', 'cloud', '--dry-run'],
    { SDKWORK_PACKAGE_VERSION: '9.8.7', SDKWORK_RELEASE_VERSION: '9.8.6' },
  );
  assert.notEqual(conflict.status, 0);
  assert.match(conflict.stderr, /SDKWORK_PACKAGE_VERSION conflicts with SDKWORK_RELEASE_VERSION/u);

  const unsupported = runNode(
    ['scripts/webserver-release.mjs', 'package', '--deployment-profile', 'cloud', '--dry-run'],
    { SDKWORK_PACKAGE_VERSION: '9.8.7', SDKWORK_PACKAGE_ARCHITECTURE: 'riscv64' },
  );
  assert.notEqual(unsupported.status, 0);
  assert.match(unsupported.stderr, /release architecture must be x64 or arm64/u);
});

test('actual Linux archive generation fails before build on a mismatched host', () => {
  const architecture = process.platform === 'linux' && process.arch === 'x64' ? 'arm64' : 'x64';
  const result = runNode(
    ['scripts/webserver-release.mjs', 'package', '--deployment-profile', 'cloud'],
    { SDKWORK_PACKAGE_VERSION: '9.8.7', SDKWORK_PACKAGE_ARCHITECTURE: architecture },
  );
  assert.notEqual(result.status, 0);
  assert.match(
    result.stderr,
    new RegExp(`linux-${architecture} server archives must be packaged on a linux-${architecture} runner`, 'u'),
  );
});

test('release smoke fails before archive access on a mismatched host architecture', () => {
  const architecture = process.platform === 'linux' && process.arch === 'x64' ? 'arm64' : 'x64';
  const result = runNode(
    [
      'scripts/webserver-release-smoke.mjs',
      '--deployment-profile',
      'standalone',
      '--architecture',
      architecture,
      '--version',
      '9.8.7',
    ],
  );
  assert.notEqual(result.status, 0);
  assert.match(
    result.stderr,
    new RegExp(`Linux ${architecture} release smoke must run on a linux-${architecture} host`, 'u'),
  );
});

test('release workflow and archive implementation preserve immutable bounded package contracts', () => {
  const workflow = readJson('sdkwork.workflow.json');
  assert.equal(workflow.lifecycle.package.length, 1);
  assert.equal(workflow.lifecycle.package[0].run, 'node scripts/webserver-release.mjs package');
  assert.deepEqual(
    workflow.targets.map((target) => target.deploymentProfile).sort(),
    ['cloud', 'cloud', 'standalone', 'standalone'],
  );
  assert.deepEqual(
    [...new Set(workflow.targets.map((target) => target.architecture))].sort(),
    ['arm64', 'x64'],
  );
  for (const target of workflow.targets) {
    assert.equal(target.platform, 'linux');
    assert.ok(['x64', 'arm64'].includes(target.architecture));
    assert.equal(target.runner, target.architecture === 'arm64' ? 'ubuntu-24.04-arm' : 'ubuntu-24.04');
    assert.deepEqual(target.formats, ['tar.gz']);
    assert.deepEqual(target.outputGlobs, [
      `dist/release/sdkwork-web-linux-${target.architecture}-${target.deploymentProfile}-server-*.tar.gz`,
      `dist/release/sdkwork-web-linux-${target.architecture}-${target.deploymentProfile}-server-*.tar.gz.sha256`,
      `dist/release/sdkwork-web-linux-${target.architecture}-${target.deploymentProfile}-server-*.tar.gz.cdx.json`,
      `dist/release/sdkwork-web-linux-${target.architecture}-${target.deploymentProfile}-server-*.tar.gz.cdx.json.sha256`,
    ]);
  }
  assert.equal(workflow.security.sbomRequired, true);
  assert.ok(
    workflow.lifecycle.sbom.some((step) => step.run === 'node scripts/webserver-sbom.mjs generate'),
  );
  assert.ok(
    workflow.lifecycle.validate.some((step) => step.run === 'node scripts/webserver-sbom.mjs validate'),
  );

  const source = readFileSync(path.join(REPO_ROOT, 'scripts/webserver-release.mjs'), 'utf8');
  assert.match(source, /MAX_ARCHIVE_BYTES = 512 \* 1024 \* 1024/u);
  assert.match(source, /function resolveCargoTargetRoot\(\)/u);
  assert.match(source, /CARGO_TARGET_DIR/u);
  assert.match(source, /path\.join\(cargoTargetRoot, 'release', binary\)/u);
  assert.match(source, /SDKWORK_PACKAGE_VERSION/u);
  assert.match(source, /SOURCE_DATE_EPOCH/u);
  for (const argument of ["'--sort=name'", "'--owner=0'", "'--group=0'", "'--numeric-owner'"]) {
    assert.match(source, new RegExp(argument, 'u'));
  }
  assert.match(source, /package\.manifest\.json/u);
  assert.match(source, /sha256File\(archive\)/u);
  assert.match(source, /renameSync\(temporaryArchive, archive\)/u);
  assert.match(source, /SUPPORTED_ARCHITECTURES = new Set\(\['x64', 'arm64'\]\)/u);
  assert.match(source, /process\.platform !== 'linux' \|\| process\.arch !== architecture/u);
  assert.ok(source.indexOf("process.platform !== 'linux'") < source.indexOf('ensureCriticalSources();'));
  assert.match(source, /source: 'etc\/examples\/public\/index\.html'/u);
  assert.match(source, /target: 'etc\/node-daemon\/development\.env\.example'/u);
});
