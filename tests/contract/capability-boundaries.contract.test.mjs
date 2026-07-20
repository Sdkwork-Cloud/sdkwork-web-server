import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { parse as parseYaml } from 'yaml';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function filesBelow(relativeRoot, predicate = () => true) {
  const root = path.join(ROOT, relativeRoot);
  const files = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        if (!['generated', 'node_modules', 'target'].includes(entry.name)) {
          visit(absolute);
        }
      } else if (predicate(absolute)) {
        files.push(absolute);
      }
    }
  };
  visit(root);
  return files;
}

function relative(file) {
  return path.relative(ROOT, file).replaceAll('\\', '/');
}

test('application-owned APIs and dependencies cannot bypass SDKWork Drive', () => {
  const apiFiles = filesBelow('apis', (file) => /\.ya?ml$/u.test(file));
  const forbiddenRoutes = [];
  for (const file of apiFiles) {
    const document = parseYaml(readFileSync(file, 'utf8'));
    for (const route of Object.keys(document?.paths ?? {})) {
      if (/\/(?:uploads?|upload_sessions?|presign|multipart|file_parts?)(?:\/|\{|$)/iu.test(route)) {
        forbiddenRoutes.push(`${relative(file)}:${route}`);
      }
    }
  }
  assert.deepEqual(
    forbiddenRoutes,
    [],
    `business upload lifecycle routes must be owned by sdkwork-drive: ${forbiddenRoutes.join(', ')}`,
  );

  const manifestFiles = [
    path.join(ROOT, 'Cargo.toml'),
    path.join(ROOT, 'package.json'),
    ...filesBelow('crates', (file) => path.basename(file) === 'Cargo.toml'),
    ...filesBelow('apps', (file) => ['Cargo.toml', 'package.json'].includes(path.basename(file))),
  ];
  const directProviderPattern = /(?:aws-sdk-s3|aws_sdk_s3|rusoto_s3|@aws-sdk\/client-s3|aliyun[-_].*oss|minio)/iu;
  const providerDependencies = manifestFiles
    .filter((file) => directProviderPattern.test(readFileSync(file, 'utf8')))
    .map(relative);
  assert.deepEqual(
    providerDependencies,
    [],
    `direct storage provider dependencies are forbidden; integrate sdkwork-drive: ${providerDependencies.join(', ')}`,
  );

  const rustSources = filesBelow('crates', (file) => file.endsWith('.rs') && !relative(file).includes('/tests/'));
  const rawDriveCalls = rustSources
    .filter((file) => /\/app\/v3\/api\/drive\/(?:uploader|upload_sessions?)/u.test(readFileSync(file, 'utf8')))
    .map(relative);
  assert.deepEqual(
    rawDriveCalls,
    [],
    `trusted Rust backends must use DriveUploaderService instead of Drive App API HTTP: ${rawDriveCalls.join(', ')}`,
  );
});

test('introducing Rust RPC requires SDKWork RPC framework and discovery together', () => {
  const cargoFiles = [
    path.join(ROOT, 'Cargo.toml'),
    ...filesBelow('crates', (file) => path.basename(file) === 'Cargo.toml'),
  ];
  const cargoText = cargoFiles.map((file) => readFileSync(file, 'utf8')).join('\n');
  const hasRpcTransport = /^(?:tonic|prost|grpcio)\s*(?:=|\.)/mu.test(cargoText);

  if (!hasRpcTransport) {
    assert.doesNotMatch(cargoText, /sdkwork-rpc-discovery/u);
    return;
  }

  assert.match(
    cargoText,
    /sdkwork-rpc-(?:server|client)/u,
    'RPC transport requires sdkwork-rpc-framework server/client integration',
  );
  assert.match(
    cargoText,
    /sdkwork-rpc-discovery/u,
    'RPC transport requires sdkwork-discovery integration through sdkwork-rpc-discovery',
  );
});

test('Web Server runtime configuration does not retain cross-application IM ownership', () => {
  const activeFiles = [
    path.join(ROOT, 'package.json'),
    path.join(ROOT, 'etc', 'sdkwork.deployment.config.json'),
    ...filesBelow('scripts', (file) => /\.(?:mjs|js|ts)$/u.test(file)),
  ];
  const stale = activeFiles
    .filter((file) => /sdkwork-im|im-dev|dev-im-ingress/iu.test(readFileSync(file, 'utf8')))
    .map(relative);
  assert.deepEqual(stale, [], `stale sdkwork-im development ownership remains: ${stale.join(', ')}`);
});
