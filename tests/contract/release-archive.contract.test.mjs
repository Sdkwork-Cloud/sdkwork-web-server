import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { create } from 'tar';
import { parse as parseYaml } from 'yaml';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const OUTPUT_ROOT = path.join(REPO_ROOT, 'dist', 'release');
const ARCHIVE_DIRECTORIES = [
  'sdkwork-web',
  'sdkwork-web/bin',
  'sdkwork-web/etc',
  'sdkwork-web/etc/examples',
  'sdkwork-web/etc/examples/public',
  'sdkwork-web/etc/node-daemon',
  'sdkwork-web/specs',
];
const PACKAGE_FILES = new Map([
  ['bin/sdkwork-web-standalone-gateway', 'gateway fixture\n'],
  ['bin/sdkwork-web-node-daemon', 'canonical node daemon fixture\n'],
  ['bin/sdkwork-web-agent', 'node daemon fixture\n'],
  ['bin/sdkwork-webserver-certificate-worker', 'certificate worker fixture\n'],
  ['sdkwork.app.config.json', '{}\n'],
  ['specs/sdkwork.webserver.config.schema.json', '{}\n'],
  ['etc/examples/sdkwork.webserver.config.json', '{}\n'],
  ['etc/examples/public/index.html', '<h1>release fixture</h1>\n'],
  ['etc/node-daemon/development.env.example', 'SDKWORK_WEB_NODE_TOKEN=\n'],
]);

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function runValidator(profile, version, architecture = 'x64') {
  const env = { ...process.env };
  delete env.SDKWORK_PACKAGE_VERSION;
  delete env.SDKWORK_RELEASE_VERSION;
  delete env.SDKWORK_PACKAGE_ARCHITECTURE;
  return spawnSync(
    process.execPath,
    [
      'scripts/webserver-release.mjs',
      'validate',
      '--deployment-profile',
      profile,
      '--architecture',
      architecture,
      '--version',
      version,
    ],
    { cwd: REPO_ROOT, encoding: 'utf8', env, windowsHide: true },
  );
}

function runSbom(operation, profile, version, architecture = 'x64') {
  const env = { ...process.env };
  delete env.SDKWORK_PACKAGE_VERSION;
  delete env.SDKWORK_RELEASE_VERSION;
  delete env.SDKWORK_PACKAGE_ARCHITECTURE;
  return spawnSync(
    process.execPath,
    [
      'scripts/webserver-sbom.mjs',
      operation,
      '--deployment-profile',
      profile,
      '--architecture',
      architecture,
      '--version',
      version,
    ],
    { cwd: REPO_ROOT, encoding: 'utf8', env, windowsHide: true },
  );
}

async function createFixture(options) {
  const profile = options.profile ?? 'standalone';
  const architecture = options.architecture ?? 'x64';
  const version = options.version;
  const artifactBase = `sdkwork-web-linux-${architecture}-${profile}-server-${version}`;
  const archive = path.join(OUTPUT_ROOT, `${artifactBase}.tar.gz`);
  const checksum = `${archive}.sha256`;
  const temporaryRoot = mkdtempSync(path.join(tmpdir(), 'sdkwork-web-release-fixture-'));
  const stageRoot = path.join(temporaryRoot, 'sdkwork-web');
  const content = [];

  for (const [relativePath, text] of PACKAGE_FILES) {
    const filePath = path.join(stageRoot, ...relativePath.split('/'));
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, text, 'utf8');
    chmodSync(
      filePath,
      relativePath.startsWith('bin/') ? (options.binaryMode ?? 0o755) : 0o644,
    );
    const bytes = readFileSync(filePath);
    content.push({ path: relativePath, bytes: bytes.length, sha256: sha256(bytes) });
  }
  content.sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0));
  const manifest = {
    schemaVersion: 1,
    kind: 'sdkwork.server-package',
    application: 'sdkwork-web',
    version,
    deploymentProfile: profile,
    runtimeTarget: 'server',
    platform: 'linux',
    architecture,
    sourceDateEpoch: 0,
    content,
  };
  options.mutateManifest?.(manifest);
  const manifestPath = path.join(stageRoot, 'package.manifest.json');
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  chmodSync(manifestPath, 0o644);

  const additionalEntries = [];
  if (options.extraFile) {
    const extraPath = path.join(stageRoot, 'etc', 'unexpected.txt');
    writeFileSync(extraPath, 'unexpected\n', 'utf8');
    chmodSync(extraPath, 0o644);
    additionalEntries.push('sdkwork-web/etc/unexpected.txt');
  }
  if (options.symbolicLink) {
    const linkPath = path.join(stageRoot, 'etc', 'unsafe-link');
    symlinkSync('../sdkwork.app.config.json', linkPath, 'file');
    additionalEntries.push('sdkwork-web/etc/unsafe-link');
  }
  const entries = [
    ...ARCHIVE_DIRECTORIES,
    'sdkwork-web/package.manifest.json',
    ...content.map((item) => `sdkwork-web/${item.path}`),
    ...additionalEntries,
  ].sort();
  if (options.duplicateEntry) {
    entries.push('sdkwork-web/sdkwork.app.config.json');
  }

  mkdirSync(OUTPUT_ROOT, { recursive: true });
  rmSync(archive, { force: true });
  rmSync(checksum, { force: true });
  await create(
    {
      file: archive,
      cwd: temporaryRoot,
      gzip: { portable: true },
      portable: false,
      mtime: new Date(0),
      noDirRecurse: true,
      filter(entryPath, stat) {
        stat.uid = 0;
        stat.gid = 0;
        stat.mode = entryPath.startsWith('sdkwork-web/bin/')
          ? 0o100000 | (options.binaryMode ?? 0o755)
          : stat.isDirectory()
            ? 0o040755
            : 0o100644;
        return true;
      },
    },
    entries,
  );
  const archiveBytes = readFileSync(archive);
  writeFileSync(checksum, `${sha256(archiveBytes)}  ${path.basename(archive)}\n`, 'utf8');
  rmSync(temporaryRoot, { recursive: true, force: true });
  return {
    archive,
    checksum,
    cleanup() {
      rmSync(archive, { force: true });
      rmSync(checksum, { force: true });
      rmSync(`${archive}.cdx.json`, { force: true });
      rmSync(`${archive}.cdx.json.sha256`, { force: true });
    },
  };
}

test('workspace and workflow close the frozen release dependency graph', () => {
  const packageJson = JSON.parse(readFileSync(path.join(REPO_ROOT, 'package.json'), 'utf8'));
  const workspace = parseYaml(readFileSync(path.join(REPO_ROOT, 'pnpm-workspace.yaml'), 'utf8'));
  const lockfile = parseYaml(readFileSync(path.join(REPO_ROOT, 'pnpm-lock.yaml'), 'utf8'));
  const workflow = JSON.parse(readFileSync(path.join(REPO_ROOT, 'sdkwork.workflow.json'), 'utf8'));
  const thinWorkflow = readFileSync(path.join(REPO_ROOT, '.github/workflows/package.yml'), 'utf8');

  assert.equal(packageJson.dependencies.tar, '7.5.20');
  assert.equal(packageJson.dependencies['@sdkwork/app-topology'], undefined);
  assert.ok(workspace.packages.includes('../sdkwork-sdk-commons/sdkwork-sdk-common-typescript'));
  assert.equal(lockfile.importers['.'].dependencies.tar.specifier, '7.5.20');
  assert.equal(lockfile.importers['.'].dependencies['@sdkwork/app-topology'], undefined);
  const dependencyIds = new Set(workflow.dependencies.map((dependency) => dependency.id));
  for (const dependencyId of ['sdkwork-core', 'sdkwork-ui', 'sdkwork-sdk-commons']) {
    assert.ok(dependencyIds.has(dependencyId));
  }
  for (const ref of ['SDKWORK_CORE_REF', 'SDKWORK_UI_REF', 'SDKWORK_SDK_COMMONS_REF']) {
    assert.match(thinWorkflow, new RegExp(ref, 'u'));
  }
  assert.ok(
    workflow.lifecycle.validate.some(
      (step) => step.run === 'node scripts/webserver-release.mjs validate',
    ),
  );
  const archiveValidationIndex = workflow.lifecycle.validate.findIndex(
    (step) => step.run === 'node scripts/webserver-release.mjs validate',
  );
  const runtimeSmokeIndex = workflow.lifecycle.validate.findIndex(
    (step) => step.run === 'node scripts/webserver-release-smoke.mjs',
  );
  assert.ok(archiveValidationIndex >= 0 && runtimeSmokeIndex > archiveValidationIndex);
  assert.equal(workflow.security.sbomRequired, true);
  assert.ok(
    workflow.lifecycle.sbom.some((step) => step.run === 'node scripts/webserver-sbom.mjs generate'),
  );
  const sbomValidationIndex = workflow.lifecycle.validate.findIndex(
    (step) => step.run === 'node scripts/webserver-sbom.mjs validate',
  );
  assert.ok(sbomValidationIndex > runtimeSmokeIndex);
});

test('Linux release smoke validates, extracts, serves HTTP and HTTPS, and cleans up', () => {
  const packageJson = JSON.parse(readFileSync(path.join(REPO_ROOT, 'package.json'), 'utf8'));
  assert.equal(
    packageJson.scripts['release:smoke:standalone'],
    'node scripts/webserver-release-smoke.mjs --deployment-profile standalone',
  );
  assert.equal(
    packageJson.scripts['release:smoke:cloud'],
    'node scripts/webserver-release-smoke.mjs --deployment-profile cloud',
  );
  const source = readFileSync(
    path.join(REPO_ROOT, 'scripts/webserver-release-smoke.mjs'),
    'utf8',
  );
  assert.match(source, /SUPPORTED_ARCHITECTURES = new Set\(\['x64', 'arm64'\]\)/u);
  assert.match(source, /process\.platform !== 'linux' \|\| process\.arch !== resolved\.architecture/u);
  assert.match(source, /scripts\/webserver-release\.mjs/u);
  assert.match(source, /extractTar/u);
  assert.match(source, /preservePaths: false/u);
  assert.match(source, /openssl/u);
  assert.match(source, /\['data-plane', smokeConfigPath\]/u);
  assert.match(source, /waitForHealth\('http'/u);
  assert.match(source, /waitForHealth\('https'/u);
  assert.match(source, /child\.kill\('SIGTERM'\)/u);
  assert.match(source, /rmSync\(temporaryRoot, \{ recursive: true, force: true \}\)/u);
});

test('bounded release validator accepts an exact immutable archive', async () => {
  const version = '9.8.7-valid';
  const fixture = await createFixture({ version });
  try {
    const result = runValidator('standalone', version);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /validated artifact=.* bytes=[0-9]+ entries=17/u);
  } finally {
    fixture.cleanup();
  }
});

test('CycloneDX SBOM binds the archive and locked Cargo closure and rejects semantic tampering', async () => {
  const version = '9.8.7-sbom';
  const fixture = await createFixture({ version });
  const sbomPath = `${fixture.archive}.cdx.json`;
  const checksumPath = `${sbomPath}.sha256`;
  try {
    const missing = runSbom('validate', 'standalone', version);
    assert.notEqual(missing.status, 0);
    assert.match(missing.stderr, /release SBOM does not exist/u);

    const generated = runSbom('generate', 'standalone', version);
    assert.equal(generated.status, 0, generated.stderr);
    const sbom = JSON.parse(readFileSync(sbomPath, 'utf8'));
    assert.equal(sbom.bomFormat, 'CycloneDX');
    assert.equal(sbom.specVersion, '1.6');
    assert.equal(sbom.metadata.component.version, version);
    assert.equal(sbom.metadata.component.hashes[0].content, sha256(readFileSync(fixture.archive)));
    assert.ok(sbom.components.length > 0 && sbom.components.length <= 20_000);
    for (const packageName of [
      'sdkwork-web-agent',
      'sdkwork-web-standalone-gateway',
      'sdkwork-webserver-certificate-worker',
    ]) {
      assert.ok(sbom.components.some((component) => component.name === packageName));
    }
    const valid = runSbom('validate', 'standalone', version);
    assert.equal(valid.status, 0, valid.stderr);

    sbom.metadata.component.name = 'sdkwork-web-tampered';
    const tamperedText = `${JSON.stringify(sbom, null, 2)}\n`;
    writeFileSync(sbomPath, tamperedText, 'utf8');
    writeFileSync(checksumPath, `${sha256(tamperedText)}  ${path.basename(sbomPath)}\n`, 'utf8');
    const tampered = runSbom('validate', 'standalone', version);
    assert.notEqual(tampered.status, 0);
    assert.match(
      tampered.stderr,
      /release SBOM does not match the artifact and locked Cargo dependency closure/u,
    );
  } finally {
    fixture.cleanup();
  }
});

test('bounded release validator binds an arm64 archive to its manifest architecture', async () => {
  const version = '9.8.7-arm64';
  const fixture = await createFixture({ version, architecture: 'arm64' });
  try {
    const result = runValidator('standalone', version, 'arm64');
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /validated artifact=sdkwork-web-linux-arm64-/u);

    const wrongArchitecture = runValidator('standalone', version, 'x64');
    assert.notEqual(wrongArchitecture.status, 0);
    assert.match(
      wrongArchitecture.stderr,
      /sdkwork-web-linux-x64-standalone-server-9\.8\.7-arm64\.tar\.gz/u,
    );
  } finally {
    fixture.cleanup();
  }
});

test('bounded release validator rejects a relabelled arm64 manifest', async () => {
  const version = '9.8.7-arm64-relabelled';
  const fixture = await createFixture({
    version,
    architecture: 'arm64',
    mutateManifest(manifest) {
      manifest.architecture = 'x64';
    },
  });
  try {
    const result = runValidator('standalone', version, 'arm64');
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /package manifest identity does not match the selected artifact/u);
  } finally {
    fixture.cleanup();
  }
});

test('release validator rejects a checksum sidecar mismatch before archive trust', async () => {
  const version = '9.8.7-checksum';
  const fixture = await createFixture({ version });
  try {
    writeFileSync(fixture.checksum, `${'0'.repeat(64)}  ${path.basename(fixture.archive)}\n`);
    const result = runValidator('standalone', version);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /SHA-256 does not match its sidecar/u);
  } finally {
    fixture.cleanup();
  }
});

test('release validator rejects manifest hashes that do not match streamed content', async () => {
  const version = '9.8.7-manifest';
  const fixture = await createFixture({
    version,
    mutateManifest(manifest) {
      manifest.content[0].sha256 = '0'.repeat(64);
    },
  });
  try {
    const result = runValidator('standalone', version);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /package content does not match manifest/u);
  } finally {
    fixture.cleanup();
  }
});

test('release validator rejects duplicate and unexpected archive entries', async () => {
  for (const [suffix, options, message] of [
    ['duplicate', { duplicateEntry: true }, /duplicate entry/u],
    ['extra', { extraFile: true }, /archive file inventory/u],
  ]) {
    const version = `9.8.7-${suffix}`;
    const fixture = await createFixture({ version, ...options });
    try {
      const result = runValidator('standalone', version);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, message);
    } finally {
      fixture.cleanup();
    }
  }
});

test('release validator requires executable Linux server binaries', async () => {
  const version = '9.8.7-mode';
  const fixture = await createFixture({ version, binaryMode: 0o644 });
  try {
    const result = runValidator('standalone', version);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /archive binary .* must be executable/u);
  } finally {
    fixture.cleanup();
  }
});

test(
  'release validator rejects symbolic-link archive entries',
  { skip: process.platform === 'win32' ? 'tar cannot create a portable NTFS symlink fixture' : false },
  async () => {
    const version = '9.8.7-symlink';
    const fixture = await createFixture({ version, symbolicLink: true });
    try {
      const result = runValidator('standalone', version);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /unsupported metadata or links|unsupported type SymbolicLink/u);
    } finally {
      fixture.cleanup();
    }
  },
);
