import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
export const DEFAULT_IM_DEV_CONFIG_PATH = path.join(
  REPO_ROOT,
  'etc',
  'sdkwork.webserver.im-dev.json',
);

function assertObject(value, label) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value;
}

function assertExactKeys(value, allowedKeys, label) {
  for (const key of Object.keys(value)) {
    if (!allowedKeys.includes(key)) {
      throw new Error(`${label} contains unsupported field ${key}`);
    }
  }
}

function validatePort(value, label) {
  if (!Number.isInteger(value) || value < 1 || value > 65535) {
    throw new Error(`${label} must be an integer between 1 and 65535`);
  }
  return value;
}

function validateHost(value, label) {
  if (typeof value !== 'string' || !value || /[\s/?#\\]/u.test(value)) {
    throw new Error(`${label} must be a non-empty host without URL syntax`);
  }
  return value;
}

function resolveConfigRelativePath(configDirectory, value, label) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error(`${label} must be a non-empty path`);
  }
  return path.resolve(configDirectory, value);
}

function validateApplication(configDirectory, value, label) {
  const application = assertObject(value, label);
  assertExactKeys(application, ['root', 'host', 'port'], label);
  const root = resolveConfigRelativePath(configDirectory, application.root, `${label}.root`);
  if (!existsSync(path.join(root, 'package.json'))) {
    throw new Error(`${label}.root does not contain package.json: ${root}`);
  }
  return {
    root,
    host: validateHost(application.host, `${label}.host`),
    port: validatePort(application.port, `${label}.port`),
  };
}

function validateCertificate(configDirectory, value) {
  const certificate = assertObject(value, 'https.certificate');
  if (certificate.mode === 'auto') {
    assertExactKeys(certificate, ['mode', 'directory'], 'https.certificate');
    return {
      mode: 'auto',
      directory: resolveConfigRelativePath(
        configDirectory,
        certificate.directory,
        'https.certificate.directory',
      ),
    };
  }
  if (certificate.mode === 'files') {
    assertExactKeys(
      certificate,
      ['mode', 'certificateFile', 'privateKeyFile'],
      'https.certificate',
    );
    return {
      mode: 'files',
      certificateFile: resolveConfigRelativePath(
        configDirectory,
        certificate.certificateFile,
        'https.certificate.certificateFile',
      ),
      privateKeyFile: resolveConfigRelativePath(
        configDirectory,
        certificate.privateKeyFile,
        'https.certificate.privateKeyFile',
      ),
    };
  }
  throw new Error('https.certificate.mode must be auto or files');
}

function validatePathPrefix(value) {
  if (
    typeof value !== 'string'
    || !value.startsWith('/')
    || !value.endsWith('/')
    || value.includes('?')
    || value.includes('#')
    || /[\u0000-\u001f\\]/u.test(value)
  ) {
    throw new Error('route.pathPrefix must start and end with / and contain no query or fragment');
  }
  return value;
}

function validateMobileTokens(value) {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new Error('route.mobileUserAgentTokens must contain between 1 and 64 tokens');
  }
  return value.map((token, index) => {
    if (typeof token !== 'string' || !token || token.length > 64 || /[\u0000-\u001f]/u.test(token)) {
      throw new Error(`route.mobileUserAgentTokens[${index}] is invalid`);
    }
    return token;
  });
}

export function resolveImDevConfigPath(value, cwd = process.cwd()) {
  return value ? path.resolve(cwd, value) : DEFAULT_IM_DEV_CONFIG_PATH;
}

export function loadImDevConfig(configPath = DEFAULT_IM_DEV_CONFIG_PATH) {
  const resolvedConfigPath = path.resolve(configPath);
  let source;
  try {
    source = JSON.parse(readFileSync(resolvedConfigPath, 'utf8'));
  } catch (error) {
    throw new Error(`cannot read IM dev config ${resolvedConfigPath}: ${error.message}`);
  }
  const config = assertObject(source, 'IM dev config');
  assertExactKeys(
    config,
    ['$schema', 'schemaVersion', 'kind', 'route', 'https', 'applications'],
    'IM dev config',
  );
  if (config.schemaVersion !== 1) {
    throw new Error('IM dev config schemaVersion must be 1');
  }
  if (config.kind !== 'sdkwork.webserver.im-dev') {
    throw new Error('IM dev config kind must be sdkwork.webserver.im-dev');
  }

  const configDirectory = path.dirname(resolvedConfigPath);
  const route = assertObject(config.route, 'route');
  assertExactKeys(route, ['pathPrefix', 'mobileUserAgentTokens'], 'route');
  const https = assertObject(config.https, 'https');
  assertExactKeys(https, ['host', 'port', 'certificate'], 'https');
  const applications = assertObject(config.applications, 'applications');
  assertExactKeys(applications, ['pc', 'h5'], 'applications');

  return {
    configPath: resolvedConfigPath,
    route: {
      pathPrefix: validatePathPrefix(route.pathPrefix),
      mobileUserAgentTokens: validateMobileTokens(route.mobileUserAgentTokens),
    },
    https: {
      host: validateHost(https.host, 'https.host'),
      port: validatePort(https.port, 'https.port'),
      certificate: validateCertificate(configDirectory, https.certificate),
    },
    applications: {
      pc: validateApplication(configDirectory, applications.pc, 'applications.pc'),
      h5: validateApplication(configDirectory, applications.h5, 'applications.h5'),
    },
  };
}
