#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  closeSync,
  copyFileSync,
  existsSync,
  fsyncSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { list as listTar } from 'tar';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const STAGE_PARENT = path.join(REPO_ROOT, '.sdkwork', 'runtime', 'release-stage');
const OUTPUT_ROOT = path.join(REPO_ROOT, 'dist', 'release');
const MAX_ARCHIVE_BYTES = 512 * 1024 * 1024;
const MAX_PACKAGE_FILE_BYTES = 256 * 1024 * 1024;
const MAX_PACKAGE_CONTENT_BYTES = 1024 * 1024 * 1024;
const MAX_PACKAGE_ENTRIES = 64;
const MAX_MANIFEST_BYTES = 256 * 1024;
const MAX_CHECKSUM_BYTES = 256;
const HASH_BUFFER_BYTES = 64 * 1024;
const PROCESS_OUTPUT_BYTES = 1024 * 1024;
const GIT_TIMEOUT_MS = 30 * 1000;
const TAR_TIMEOUT_MS = 5 * 60 * 1000;
const CARGO_BUILD_TIMEOUT_MS = 30 * 60 * 1000;
const SBOM_TIMEOUT_MS = 3 * 60 * 1000;
const SUPPORTED_ARCHITECTURES = new Set(['x64', 'arm64']);
const BINARIES = [
  'sdkwork-api-web-server-standalone-gateway',
  'sdkwork-web-server-website-delivery-edge-runtime',
  'sdkwork-web-node-daemon',
  'sdkwork-web-agent',
  'sdkwork-webserver-certificate-worker',
];
const PACKAGE_ASSETS = [
  { source: 'sdkwork.app.config.json', target: 'sdkwork.app.config.json' },
  {
    source: 'specs/sdkwork.webserver.config.schema.json',
    target: 'specs/sdkwork.webserver.config.schema.json',
  },
  {
    source: 'etc/examples/sdkwork.webserver.config.json',
    target: 'etc/examples/sdkwork.webserver.config.json',
  },
  {
    source: 'etc/examples/public/index.html',
    target: 'etc/examples/public/index.html',
  },
  {
    source: 'etc/data-plane/website.cloud.config.json',
    target: 'etc/data-plane/website.cloud.config.json',
  },
  {
    source: 'etc/node-daemon/development.env.example',
    target: 'etc/node-daemon/development.env.example',
  },
  { source: 'database/README.md', target: 'database/README.md' },
  {
    source: 'database/database.manifest.json',
    target: 'database/database.manifest.json',
  },
  {
    source: 'database/contract/prefix-registry.json',
    target: 'database/contract/prefix-registry.json',
  },
  {
    source: 'database/contract/schema.yaml',
    target: 'database/contract/schema.yaml',
  },
  {
    source: 'database/contract/table-registry.json',
    target: 'database/contract/table-registry.json',
  },
  {
    source: 'database/ddl/baseline/postgres/0001_web_baseline.sql',
    target: 'database/ddl/baseline/postgres/0001_web_baseline.sql',
  },
  {
    source: 'database/ddl/baseline/sqlite/0001_web_baseline.sql',
    target: 'database/ddl/baseline/sqlite/0001_web_baseline.sql',
  },
  {
    source: 'database/drift/policy.yaml',
    target: 'database/drift/policy.yaml',
  },
  {
    source: 'database/seeds/seed.manifest.json',
    target: 'database/seeds/seed.manifest.json',
  },
  {
    source: 'database/seeds/common/001_bootstrap.sql',
    target: 'database/seeds/common/001_bootstrap.sql',
  },
];
const EXPECTED_CONTENT_PATHS = [
  ...BINARIES.map((binary) => `bin/${binary}`),
  ...PACKAGE_ASSETS.map((asset) => asset.target),
].sort();
const EXPECTED_ARCHIVE_DIRECTORIES = Array.from(
  new Set(
    EXPECTED_CONTENT_PATHS.flatMap((contentPath) => {
      const segments = contentPath.split('/');
      return segments.slice(0, -1).map((_, index) =>
        ['sdkwork-web', ...segments.slice(0, index + 1)].join('/'),
      );
    }).concat('sdkwork-web'),
  ),
).sort();

function parseArgs(argv) {
  const settings = {
    operation: argv[0],
    deploymentProfile: process.env.SDKWORK_DEPLOYMENT_PROFILE,
    architecture: process.env.SDKWORK_PACKAGE_ARCHITECTURE,
    version: undefined,
    dryRun: false,
  };
  for (let index = 1; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--deployment-profile') {
      settings.deploymentProfile = argv[++index];
    } else if (argument === '--architecture') {
      settings.architecture = argv[++index];
    } else if (argument === '--version') {
      settings.version = argv[++index];
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

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? REPO_ROOT,
    encoding: 'utf8',
    env: options.env ?? process.env,
    stdio: options.capture ? 'pipe' : 'inherit',
    timeout: options.timeoutMs,
    maxBuffer: PROCESS_OUTPUT_BYTES,
    killSignal: 'SIGKILL',
    windowsHide: true,
  });
  if (result.error || result.status !== 0) {
    const detail = result.error?.message ?? result.stderr?.trim() ?? `exit ${result.status}`;
    throw new Error(`${command} ${args.join(' ')} failed: ${detail}`);
  }
  return result;
}

function assertExactKeys(value, keys, label) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw new Error(`${label} must contain exactly: ${expected.join(', ')}`);
  }
}

function assertSafeOwnedPath(candidate, owner, label) {
  const relative = path.relative(owner, candidate);
  if (relative.startsWith('..') || path.isAbsolute(relative) || relative === '') {
    throw new Error(`${label} is outside its owned directory`);
  }
}

function assertRegularOrMissing(filePath, label) {
  if (!existsSync(filePath)) {
    return;
  }
  const stat = lstatSync(filePath);
  if (stat.isSymbolicLink() || !stat.isFile()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
}

function inspectRegularFile(filePath, label, maxBytes = MAX_PACKAGE_FILE_BYTES) {
  const linkStat = lstatSync(filePath);
  const stat = statSync(filePath);
  if (linkStat.isSymbolicLink() || !linkStat.isFile() || !stat.isFile()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
  if (!Number.isSafeInteger(stat.size) || stat.size < 0 || stat.size > maxBytes) {
    throw new Error(`${label} must be within 0..=${maxBytes} bytes`);
  }
  return stat;
}

function syncFile(filePath) {
  const descriptor = openSync(filePath, 'r');
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}

function syncDirectory(directoryPath) {
  if (process.platform === 'win32') {
    return;
  }
  const descriptor = openSync(directoryPath, 'r');
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}

function writeAtomicText(filePath, content) {
  assertRegularOrMissing(filePath, `output ${filePath}`);
  const temporaryPath = `${filePath}.tmp-${process.pid}`;
  rmSync(temporaryPath, { force: true });
  try {
    writeFileSync(temporaryPath, content, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
    syncFile(temporaryPath);
    renameSync(temporaryPath, filePath);
    syncDirectory(path.dirname(filePath));
  } finally {
    rmSync(temporaryPath, { force: true });
  }
}

function sha256File(filePath) {
  const descriptor = openSync(filePath, 'r');
  const buffer = Buffer.allocUnsafe(HASH_BUFFER_BYTES);
  const hash = createHash('sha256');
  try {
    while (true) {
      const bytesRead = readSync(descriptor, buffer, 0, buffer.length, null);
      if (bytesRead === 0) {
        break;
      }
      hash.update(buffer.subarray(0, bytesRead));
    }
  } finally {
    closeSync(descriptor);
  }
  return hash.digest('hex');
}

function readSmallText(filePath, maxBytes, label) {
  const stat = inspectRegularFile(filePath, label, maxBytes);
  if (stat.size === 0) {
    throw new Error(`${label} must not be empty`);
  }
  return readFileSync(filePath, 'utf8');
}

function ensureCriticalSources() {
  for (const relativePath of [
    'Cargo.toml',
    'scripts/webserver-sbom.mjs',
    ...PACKAGE_ASSETS.map((asset) => asset.source),
  ]) {
    const absolutePath = path.join(REPO_ROOT, relativePath);
    if (existsSync(absolutePath)) {
      inspectRegularFile(absolutePath, `package source ${relativePath}`);
      continue;
    }
    const tracked = run('git', ['ls-files', '--error-unmatch', '--', relativePath], {
      capture: true,
      timeoutMs: GIT_TIMEOUT_MS,
    });
    if (!tracked.stdout.trim()) {
      throw new Error(`missing untracked package source ${relativePath}`);
    }
    run('git', ['checkout', 'HEAD', '--', relativePath], {
      capture: true,
      timeoutMs: GIT_TIMEOUT_MS,
    });
    if (!existsSync(absolutePath)) {
      throw new Error(`failed to recover package source ${relativePath}`);
    }
    inspectRegularFile(absolutePath, `package source ${relativePath}`);
  }
}

function resolveVersion(settings) {
  const manifestPath = path.join(REPO_ROOT, 'sdkwork.app.config.json');
  const manifestText = readSmallText(manifestPath, MAX_MANIFEST_BYTES, 'application manifest');
  const manifest = JSON.parse(manifestText);
  const packageVersion = process.env.SDKWORK_PACKAGE_VERSION?.trim();
  const compatibilityVersion = process.env.SDKWORK_RELEASE_VERSION?.trim();
  if (packageVersion && compatibilityVersion && packageVersion !== compatibilityVersion) {
    throw new Error('SDKWORK_PACKAGE_VERSION conflicts with SDKWORK_RELEASE_VERSION');
  }
  const version = settings.version || packageVersion || compatibilityVersion || manifest.release?.currentVersion;
  if (!/^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/u.test(version ?? '')) {
    throw new Error('release version must be an explicit semantic version');
  }
  return version;
}

function resolveArchitecture(settings) {
  const architecture = settings.architecture?.trim() || process.arch;
  if (!SUPPORTED_ARCHITECTURES.has(architecture)) {
    throw new Error('release architecture must be x64 or arm64');
  }
  return architecture;
}

function resolveArtifact(settings) {
  if (!['standalone', 'cloud'].includes(settings.deploymentProfile)) {
    throw new Error('--deployment-profile must be standalone or cloud');
  }
  const version = resolveVersion(settings);
  const architecture = resolveArchitecture(settings);
  const artifactBase = `sdkwork-web-linux-${architecture}-${settings.deploymentProfile}-server-${version}`;
  const archive = path.join(OUTPUT_ROOT, `${artifactBase}.tar.gz`);
  assertSafeOwnedPath(archive, OUTPUT_ROOT, 'release archive');
  return { version, architecture, artifactBase, archive };
}

function resolveCargoTargetRoot() {
  const configured = process.env.CARGO_TARGET_DIR?.trim();
  if (!configured) {
    return path.join(REPO_ROOT, 'target');
  }
  return path.isAbsolute(configured)
    ? path.normalize(configured)
    : path.resolve(REPO_ROOT, configured);
}

function copyPackageAsset(asset, stageRoot) {
  const source = path.join(REPO_ROOT, asset.source);
  inspectRegularFile(source, `package source ${asset.source}`);
  const target = path.join(stageRoot, asset.target);
  assertSafeOwnedPath(target, stageRoot, `package target ${asset.target}`);
  mkdirSync(path.dirname(target), { recursive: true, mode: 0o755 });
  copyFileSync(source, target);
  chmodSync(target, 0o644);
  return target;
}

function normalizeArchivePath(value) {
  if (typeof value !== 'string' || value.length === 0 || value.length > 512) {
    throw new Error('archive entry path must contain 1..=512 characters');
  }
  if (value.includes('\\') || value.includes('\0') || value.startsWith('/')) {
    throw new Error(`unsafe archive entry path: ${JSON.stringify(value)}`);
  }
  const normalized = value.endsWith('/') ? value.slice(0, -1) : value;
  const segments = normalized.split('/');
  if (
    normalized.length === 0 ||
    segments.some((segment) => segment === '' || segment === '.' || segment === '..') ||
    path.posix.normalize(normalized) !== normalized ||
    !normalized.startsWith('sdkwork-web') ||
    (normalized !== 'sdkwork-web' && !normalized.startsWith('sdkwork-web/'))
  ) {
    throw new Error(`unsafe archive entry path: ${JSON.stringify(value)}`);
  }
  return normalized;
}

async function inspectArchiveEntries(archive) {
  const records = new Map();
  const order = [];
  const entryCompletions = [];
  let manifestBuffer;
  let declaredBytes = 0;
  let validationError;
  const fail = (error) => {
    if (!validationError) {
      validationError = error instanceof Error ? error : new Error(String(error));
    }
  };

  await listTar({
    file: archive,
    strict: true,
    noResume: true,
    maxReadSize: HASH_BUFFER_BYTES,
    onReadEntry(entry) {
      try {
        if (records.size >= MAX_PACKAGE_ENTRIES) {
          throw new Error(`archive contains more than ${MAX_PACKAGE_ENTRIES} entries`);
        }
        const entryPath = normalizeArchivePath(entry.path);
        if (records.has(entryPath)) {
          throw new Error(`archive contains duplicate entry ${entryPath}`);
        }
        if (entry.meta || entry.invalid || entry.unsupported || entry.linkpath) {
          throw new Error(`archive entry ${entryPath} uses unsupported metadata or links`);
        }
        if (!['File', 'Directory'].includes(entry.type)) {
          throw new Error(`archive entry ${entryPath} has unsupported type ${entry.type}`);
        }
        if (entry.uid !== 0 || entry.gid !== 0) {
          throw new Error(`archive entry ${entryPath} must use uid/gid 0`);
        }
        const mode = (entry.mode ?? 0) & 0o7777;
        if ((mode & 0o022) !== 0) {
          throw new Error(`archive entry ${entryPath} must not be group/world writable`);
        }
        if (!(entry.mtime instanceof Date) || !Number.isFinite(entry.mtime.getTime())) {
          throw new Error(`archive entry ${entryPath} must have a valid mtime`);
        }
        if (!Number.isSafeInteger(entry.size) || entry.size < 0) {
          throw new Error(`archive entry ${entryPath} has an invalid size`);
        }
        if (entry.type === 'Directory' && entry.size !== 0) {
          throw new Error(`archive directory ${entryPath} must be empty metadata`);
        }
        if (entry.type === 'File' && entry.size > MAX_PACKAGE_FILE_BYTES) {
          throw new Error(`archive file ${entryPath} exceeds ${MAX_PACKAGE_FILE_BYTES} bytes`);
        }
        declaredBytes += entry.size;
        if (declaredBytes > MAX_PACKAGE_CONTENT_BYTES) {
          throw new Error(`archive content exceeds ${MAX_PACKAGE_CONTENT_BYTES} bytes`);
        }

        const record = {
          path: entryPath,
          type: entry.type,
          size: entry.size,
          mode,
          uid: entry.uid,
          gid: entry.gid,
          mtimeSeconds: Math.floor(entry.mtime.getTime() / 1000),
        };
        records.set(entryPath, record);
        order.push(entryPath);
        if (entry.type === 'Directory') {
          entry.resume();
          return;
        }

        const hash = createHash('sha256');
        let actualBytes = 0;
        const manifestChunks = [];
        const completion = new Promise((resolve, reject) => {
          entry.on('data', (chunk) => {
            actualBytes += chunk.length;
            if (actualBytes > entry.size || actualBytes > MAX_PACKAGE_FILE_BYTES) {
              fail(new Error(`archive file ${entryPath} exceeds its declared bound`));
              return;
            }
            hash.update(chunk);
            if (entryPath === 'sdkwork-web/package.manifest.json') {
              if (actualBytes > MAX_MANIFEST_BYTES) {
                fail(new Error(`package manifest exceeds ${MAX_MANIFEST_BYTES} bytes`));
                return;
              }
              manifestChunks.push(Buffer.from(chunk));
            }
          });
          entry.once('error', reject);
          entry.once('end', () => {
            record.actualBytes = actualBytes;
            record.sha256 = hash.digest('hex');
            if (entryPath === 'sdkwork-web/package.manifest.json') {
              manifestBuffer = Buffer.concat(manifestChunks, actualBytes);
            }
            resolve();
          });
        });
        entryCompletions.push(completion);
      } catch (error) {
        fail(error);
        entry.resume();
      }
    },
  });
  await Promise.all(entryCompletions);
  if (validationError) {
    throw validationError;
  }
  return { records, order, manifestBuffer };
}

function validatePackageManifest(manifestBuffer, records, order, expected) {
  if (!manifestBuffer || manifestBuffer.length === 0) {
    throw new Error('archive is missing package.manifest.json');
  }
  let manifestText;
  try {
    manifestText = new TextDecoder('utf-8', { fatal: true }).decode(manifestBuffer);
  } catch {
    throw new Error('package manifest must be valid UTF-8');
  }
  if (!manifestText.endsWith('\n')) {
    throw new Error('package manifest must end with a newline');
  }
  const manifest = JSON.parse(manifestText);
  assertExactKeys(
    manifest,
    [
      'schemaVersion',
      'kind',
      'application',
      'version',
      'deploymentProfile',
      'runtimeTarget',
      'platform',
      'architecture',
      'sourceDateEpoch',
      'content',
    ],
    'package manifest',
  );
  if (
    manifest.schemaVersion !== 1 ||
    manifest.kind !== 'sdkwork.server-package' ||
    manifest.application !== 'sdkwork-web' ||
    manifest.version !== expected.version ||
    manifest.deploymentProfile !== expected.deploymentProfile ||
    manifest.runtimeTarget !== 'server' ||
    manifest.platform !== 'linux' ||
    manifest.architecture !== expected.architecture
  ) {
    throw new Error('package manifest identity does not match the selected artifact');
  }
  if (!Number.isSafeInteger(manifest.sourceDateEpoch) || manifest.sourceDateEpoch < 0) {
    throw new Error('package manifest sourceDateEpoch must be a non-negative safe integer');
  }
  if (!Array.isArray(manifest.content) || manifest.content.length !== EXPECTED_CONTENT_PATHS.length) {
    throw new Error(`package manifest must contain exactly ${EXPECTED_CONTENT_PATHS.length} files`);
  }

  const manifestPaths = [];
  for (const [index, item] of manifest.content.entries()) {
    assertExactKeys(item, ['path', 'bytes', 'sha256'], `package manifest content[${index}]`);
    if (
      typeof item.path !== 'string' ||
      !Number.isSafeInteger(item.bytes) ||
      item.bytes < 0 ||
      item.bytes > MAX_PACKAGE_FILE_BYTES ||
      !/^[a-f0-9]{64}$/u.test(item.sha256)
    ) {
      throw new Error(`package manifest content[${index}] is invalid`);
    }
    manifestPaths.push(item.path);
    const record = records.get(`sdkwork-web/${item.path}`);
    if (
      !record ||
      record.type !== 'File' ||
      record.size !== item.bytes ||
      record.actualBytes !== item.bytes ||
      record.sha256 !== item.sha256
    ) {
      throw new Error(`package content does not match manifest for ${item.path}`);
    }
  }
  if (JSON.stringify(manifestPaths) !== JSON.stringify(EXPECTED_CONTENT_PATHS)) {
    throw new Error('package manifest content paths are missing, unexpected, duplicated, or unsorted');
  }

  const expectedFiles = [
    'sdkwork-web/package.manifest.json',
    ...EXPECTED_CONTENT_PATHS.map((item) => `sdkwork-web/${item}`),
  ].sort();
  const actualFiles = [...records.values()]
    .filter((record) => record.type === 'File')
    .map((record) => record.path)
    .sort();
  const actualDirectories = [...records.values()]
    .filter((record) => record.type === 'Directory')
    .map((record) => record.path)
    .sort();
  if (JSON.stringify(actualFiles) !== JSON.stringify(expectedFiles)) {
    throw new Error('archive file inventory does not match the package contract');
  }
  if (JSON.stringify(actualDirectories) !== JSON.stringify(EXPECTED_ARCHIVE_DIRECTORIES)) {
    throw new Error('archive directory inventory does not match the package contract');
  }
  const expectedOrder = [...expectedFiles, ...EXPECTED_ARCHIVE_DIRECTORIES].sort();
  if (JSON.stringify(order) !== JSON.stringify(expectedOrder)) {
    throw new Error('archive entries are not in deterministic path order');
  }

  for (const record of records.values()) {
    if (record.mtimeSeconds !== manifest.sourceDateEpoch) {
      throw new Error(`archive entry ${record.path} has a non-deterministic mtime`);
    }
    if (record.type === 'Directory') {
      if ((record.mode & 0o700) !== 0o700) {
        throw new Error(`archive directory ${record.path} must be owner accessible`);
      }
      continue;
    }
    if ((record.mode & 0o400) === 0) {
      throw new Error(`archive file ${record.path} must be owner readable`);
    }
    const isBinary = record.path.startsWith('sdkwork-web/bin/');
    if (isBinary && (record.mode & 0o111) === 0) {
      throw new Error(`archive binary ${record.path} must be executable`);
    }
    if (!isBinary && (record.mode & 0o111) !== 0) {
      throw new Error(`archive data file ${record.path} must not be executable`);
    }
  }
}

async function validateReleaseArchive(settings, resolved = resolveArtifact(settings)) {
  const { archive, artifactBase, version } = resolved;
  const archiveStat = inspectRegularFile(archive, 'release archive', MAX_ARCHIVE_BYTES);
  if (archiveStat.size === 0) {
    throw new Error('release archive must not be empty');
  }
  const checksumPath = `${archive}.sha256`;
  const checksumText = readSmallText(checksumPath, MAX_CHECKSUM_BYTES, 'release checksum');
  const checksumMatch = checksumText.match(/^([a-f0-9]{64})  ([^\r\n]+)\r?\n$/u);
  if (!checksumMatch || checksumMatch[2] !== path.basename(archive)) {
    throw new Error('release checksum must contain one canonical SHA-256 record');
  }
  if (sha256File(archive) !== checksumMatch[1]) {
    throw new Error('release archive SHA-256 does not match its sidecar');
  }
  const inspected = await inspectArchiveEntries(archive);
  validatePackageManifest(inspected.manifestBuffer, inspected.records, inspected.order, {
    deploymentProfile: settings.deploymentProfile,
    architecture: resolved.architecture,
    version,
  });
  console.log(
    `[sdkwork-web-release] validated artifact=${artifactBase}.tar.gz bytes=${archiveStat.size} entries=${inspected.records.size}`,
  );
}

async function packageArchive(settings) {
  const resolved = resolveArtifact(settings);
  const { version, architecture, artifactBase, archive } = resolved;
  console.log(
    `[sdkwork-web-release] operation=package deploymentProfile=${settings.deploymentProfile} runtimeTarget=server architecture=${architecture} version=${version}`,
  );
  console.log(`[sdkwork-web-release] artifact=${artifactBase}.tar.gz`);
  if (settings.dryRun) {
    return;
  }
  if (process.platform !== 'linux' || process.arch !== architecture) {
    throw new Error(
      `linux-${architecture} server archives must be packaged on a linux-${architecture} runner`,
    );
  }

  ensureCriticalSources();
  run('cargo', ['build', '--workspace', '--release'], { timeoutMs: CARGO_BUILD_TIMEOUT_MS });
  const cargoTargetRoot = resolveCargoTargetRoot();
  const stageContainer = path.join(STAGE_PARENT, `${artifactBase}-${process.pid}`);
  const stageRoot = path.join(stageContainer, 'sdkwork-web');
  assertSafeOwnedPath(stageContainer, STAGE_PARENT, 'release stage');
  rmSync(stageContainer, { recursive: true, force: true });
  mkdirSync(path.join(stageRoot, 'bin'), { recursive: true, mode: 0o755 });
  mkdirSync(OUTPUT_ROOT, { recursive: true, mode: 0o755 });

  try {
    const packagedFiles = [];
    let packageContentBytes = 0;
    for (const binary of BINARIES) {
      const source = path.join(cargoTargetRoot, 'release', binary);
      const stat = inspectRegularFile(source, `release binary ${binary}`);
      packageContentBytes += stat.size;
      const target = path.join(stageRoot, 'bin', binary);
      copyFileSync(source, target);
      chmodSync(target, 0o755);
      packagedFiles.push(target);
    }
    for (const asset of PACKAGE_ASSETS) {
      const target = copyPackageAsset(asset, stageRoot);
      packageContentBytes += statSync(target).size;
      packagedFiles.push(target);
    }
    if (packageContentBytes > MAX_PACKAGE_CONTENT_BYTES) {
      throw new Error(`package content exceeds ${MAX_PACKAGE_CONTENT_BYTES} bytes`);
    }

    const sourceDateEpoch = Number.parseInt(process.env.SOURCE_DATE_EPOCH ?? '0', 10);
    if (!Number.isSafeInteger(sourceDateEpoch) || sourceDateEpoch < 0) {
      throw new Error('SOURCE_DATE_EPOCH must be a non-negative safe integer');
    }
    const content = packagedFiles
      .map((filePath) => ({
        path: path.relative(stageRoot, filePath).split(path.sep).join('/'),
        bytes: statSync(filePath).size,
        sha256: sha256File(filePath),
      }))
      .sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0));
    const packageManifest = {
      schemaVersion: 1,
      kind: 'sdkwork.server-package',
      application: 'sdkwork-web',
      version,
      deploymentProfile: settings.deploymentProfile,
      runtimeTarget: 'server',
      platform: 'linux',
      architecture,
      sourceDateEpoch,
      content,
    };
    const packageManifestPath = path.join(stageRoot, 'package.manifest.json');
    writeFileSync(packageManifestPath, `${JSON.stringify(packageManifest, null, 2)}\n`, {
      encoding: 'utf8',
      flag: 'wx',
      mode: 0o644,
    });
    chmodSync(packageManifestPath, 0o644);

    const temporaryArchive = `${archive}.tmp-${process.pid}`;
    rmSync(temporaryArchive, { force: true });
    try {
      run(
        'tar',
        [
          '--sort=name',
          `--mtime=@${sourceDateEpoch}`,
          '--owner=0',
          '--group=0',
          '--numeric-owner',
          '-czf',
          temporaryArchive,
          'sdkwork-web',
        ],
        {
          cwd: stageContainer,
          env: { ...process.env, LC_ALL: 'C' },
          timeoutMs: TAR_TIMEOUT_MS,
        },
      );
      const archiveBytes = inspectRegularFile(
        temporaryArchive,
        'temporary release archive',
        MAX_ARCHIVE_BYTES,
      ).size;
      if (archiveBytes === 0) {
        throw new Error('release archive must not be empty');
      }
      syncFile(temporaryArchive);
      assertRegularOrMissing(archive, 'release archive');
      renameSync(temporaryArchive, archive);
      syncDirectory(OUTPUT_ROOT);
    } finally {
      rmSync(temporaryArchive, { force: true });
    }
    writeAtomicText(
      `${archive}.sha256`,
      `${sha256File(archive)}  ${path.basename(archive)}\n`,
    );
    await validateReleaseArchive(settings, resolved);
    run(
      process.execPath,
      [
        'scripts/webserver-sbom.mjs',
        'generate',
        '--deployment-profile',
        settings.deploymentProfile,
        '--architecture',
        architecture,
        '--version',
        version,
      ],
      { timeoutMs: SBOM_TIMEOUT_MS },
    );
    console.log(`[sdkwork-web-release] wrote ${path.relative(REPO_ROOT, archive)}`);
  } finally {
    rmSync(stageContainer, { recursive: true, force: true });
  }
}

async function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    console.log(
      'Usage: node scripts/webserver-release.mjs <package|validate> --deployment-profile <standalone|cloud> [--architecture <x64|arm64>] [--version <semver>] [--dry-run]',
    );
    return;
  }
  if (!['package', 'validate'].includes(settings.operation)) {
    throw new Error('operation must be package or validate');
  }
  if (settings.operation === 'package') {
    await packageArchive(settings);
    return;
  }
  const resolved = resolveArtifact(settings);
  console.log(
    `[sdkwork-web-release] operation=validate deploymentProfile=${settings.deploymentProfile} runtimeTarget=server architecture=${resolved.architecture} version=${resolved.version}`,
  );
  console.log(`[sdkwork-web-release] artifact=${resolved.artifactBase}.tar.gz`);
  if (!settings.dryRun) {
    await validateReleaseArchive(settings, resolved);
  }
}

main().catch((error) => {
  process.stderr.write(`[sdkwork-web-release] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
