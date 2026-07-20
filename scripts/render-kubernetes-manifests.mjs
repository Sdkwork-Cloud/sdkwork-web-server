#!/usr/bin/env node

import { existsSync, lstatSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SOURCE_ROOT = path.join(REPO_ROOT, 'deployments', 'kubernetes');
const OUTPUT_ROOT = path.join(REPO_ROOT, '.sdkwork', 'runtime', 'kubernetes');
const IMAGE_DIGEST_PLACEHOLDER = '__SDKWORK_IMAGE_DIGEST__';
const MANIFESTS = ['migration-job.yaml', 'deployment.yaml', 'service.yaml'];

function parseArgs(argv) {
  const settings = {};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--image-digest') {
      settings.imageDigest = argv[++index];
    } else if (argument === '--help' || argument === '-h') {
      settings.help = true;
    } else {
      throw new Error(`unsupported option: ${argument}`);
    }
  }
  return settings;
}

function normalizeDigest(value) {
  const digest = value?.trim().replace(/^sha256:/u, '').toLowerCase();
  if (!/^[a-f0-9]{64}$/u.test(digest ?? '')) {
    throw new Error('--image-digest must be a sha256 digest containing exactly 64 hex characters');
  }
  return digest;
}

function assertRegularFile(filePath, label) {
  if (!existsSync(filePath)) {
    throw new Error(`${label} is missing`);
  }
  const stat = lstatSync(filePath);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
}

function renderManifest(name, digest, outputDirectory) {
  const source = path.join(SOURCE_ROOT, name);
  assertRegularFile(source, `Kubernetes source ${name}`);
  const authored = readFileSync(source, 'utf8');
  const occurrenceCount = authored.split(IMAGE_DIGEST_PLACEHOLDER).length - 1;
  const expectedCount = name === 'service.yaml' ? 0 : 1;
  if (occurrenceCount !== expectedCount) {
    throw new Error(`${name} must contain exactly ${expectedCount} image digest placeholder(s)`);
  }
  if (/:latest(?:\s|$)/u.test(authored)) {
    throw new Error(`${name} must not contain a latest image tag`);
  }
  const rendered = authored.replaceAll(IMAGE_DIGEST_PLACEHOLDER, digest);
  const output = path.join(outputDirectory, name);
  writeFileSync(output, rendered, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
}

function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    console.log('Usage: node scripts/render-kubernetes-manifests.mjs --image-digest <sha256>');
    return;
  }
  const digest = normalizeDigest(settings.imageDigest);
  const outputDirectory = path.join(OUTPUT_ROOT, digest.slice(0, 16));
  if (existsSync(outputDirectory)) {
    throw new Error(`render output already exists: ${path.relative(REPO_ROOT, outputDirectory)}`);
  }
  mkdirSync(outputDirectory, { recursive: true, mode: 0o700 });
  for (const manifest of MANIFESTS) {
    renderManifest(manifest, digest, outputDirectory);
  }
  console.log(path.relative(REPO_ROOT, outputDirectory));
}

try {
  main();
} catch (error) {
  process.stderr.write(`[sdkwork-web-kubernetes-render] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
}
