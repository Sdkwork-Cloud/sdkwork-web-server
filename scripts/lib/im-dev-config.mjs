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
  const certificate = assertObject(value, 'listener.certificate');
  if (certificate.mode === 'auto') {
    assertExactKeys(certificate, ['mode', 'directory'], 'listener.certificate');
    return {
      mode: 'auto',
      directory: resolveConfigRelativePath(
        configDirectory,
        certificate.directory,
        'listener.certificate.directory',
      ),
    };
  }
  if (certificate.mode === 'files') {
    assertExactKeys(
      certificate,
      ['mode', 'certificateFile', 'privateKeyFile'],
      'listener.certificate',
    );
    return {
      mode: 'files',
      certificateFile: resolveConfigRelativePath(
        configDirectory,
        certificate.certificateFile,
        'listener.certificate.certificateFile',
      ),
      privateKeyFile: resolveConfigRelativePath(
        configDirectory,
        certificate.privateKeyFile,
        'listener.certificate.privateKeyFile',
      ),
    };
  }
  throw new Error('listener.certificate.mode must be auto or files');
}

function validatePathPrefix(value) {
  if (
    typeof value !== 'string'
    || !value.startsWith('/')
    || value.includes('?')
    || value.includes('#')
    || /[\u0000-\u001f\\]/u.test(value)
  ) {
    throw new Error('route.pathPrefix must start with / and contain no query or fragment');
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

function validatePathPrefixes(value, label) {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new Error(`${label} must contain between 1 and 64 path prefixes`);
  }
  const prefixes = value.map((entry, index) => validatePathPrefixEntry(entry, `${label}[${index}]`));
  if (new Set(prefixes).size !== prefixes.length) {
    throw new Error(`${label} must not contain duplicate path prefixes`);
  }
  return prefixes;
}

function validatePathPrefixEntry(value, label) {
  if (
    typeof value !== 'string'
    || !value.startsWith('/')
    || value.includes('?')
    || value.includes('#')
    || /[\u0000-\u001f\\]/u.test(value)
  ) {
    throw new Error(`${label} must start with / and contain no query or fragment`);
  }
  return value;
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
    [
      '$schema',
      'schemaVersion',
      'kind',
      'application',
      'listener',
      'route',
      'deployment',
      'applications',
    ],
    'IM dev config',
  );
  if (config.schemaVersion !== 2) {
    throw new Error('IM dev config schemaVersion must be 2');
  }
  if (config.kind !== 'sdkwork.webserver.im-dev') {
    throw new Error('IM dev config kind must be sdkwork.webserver.im-dev');
  }

  const configDirectory = path.dirname(resolvedConfigPath);
  const application = assertObject(config.application, 'application');
  assertExactKeys(
    application,
    ['manifestPath', 'deploymentConfigPath', 'environment'],
    'application',
  );
  const manifestPath = resolveConfigRelativePath(
    configDirectory,
    application.manifestPath,
    'application.manifestPath',
  );
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  } catch (error) {
    throw new Error(`cannot read application manifest ${manifestPath}: ${error.message}`);
  }
  if (manifest.schemaVersion !== 3 || manifest.kind !== 'sdkwork.app') {
    throw new Error('application manifest must be an SDKWork app manifest v3');
  }
  if (!['development', 'test', 'staging', 'production'].includes(application.environment)) {
    throw new Error('application.environment is invalid');
  }
  const deploymentConfigPath = resolveConfigRelativePath(
    configDirectory,
    application.deploymentConfigPath,
    'application.deploymentConfigPath',
  );
  let deploymentConfig;
  try {
    deploymentConfig = JSON.parse(readFileSync(deploymentConfigPath, 'utf8'));
  } catch (error) {
    throw new Error(`cannot read application deployment config ${deploymentConfigPath}: ${error.message}`);
  }
  if (
    deploymentConfig.schemaVersion !== 1
    || deploymentConfig.kind !== 'sdkwork.deployment-index'
  ) {
    throw new Error('application deployment config must be an SDKWork deployment index v1');
  }
  const environmentConfig = deploymentConfig.environments?.[application.environment];
  if (!environmentConfig || typeof environmentConfig.applicationOrigin !== 'string') {
    throw new Error(
      `application deployment config does not declare ${application.environment}.applicationOrigin`,
    );
  }
  let publicUrl;
  try {
    publicUrl = new URL(environmentConfig.applicationOrigin);
  } catch {
    throw new Error('application environment accessUrl must be an absolute URL');
  }
  if (
    !['http:', 'https:'].includes(publicUrl.protocol)
    || publicUrl.username
    || publicUrl.password
    || publicUrl.pathname !== '/'
    || publicUrl.search
    || publicUrl.hash
  ) {
    throw new Error('application environment accessUrl must be an HTTP(S) origin root');
  }
  const publicPort = publicUrl.port
    ? Number.parseInt(publicUrl.port, 10)
    : publicUrl.protocol === 'https:' ? 443 : 80;
  const listener = assertObject(config.listener, 'listener');
  assertExactKeys(listener, ['bind', 'certificate'], 'listener');
  const certificate = listener.certificate
    ? validateCertificate(configDirectory, listener.certificate)
    : undefined;
  if (publicUrl.protocol === 'https:' && !certificate) {
    throw new Error('listener.certificate is required for an HTTPS application accessUrl');
  }
  const route = assertObject(config.route, 'route');
  assertExactKeys(route, ['pathPrefix', 'mobileUserAgentTokens'], 'route');
  const deployment = assertObject(config.deployment, 'deployment');
  assertExactKeys(
    deployment,
    ['profile', 'applicationRoot', 'serverScript', 'gateway'],
    'deployment',
  );
  if (!['standalone', 'cloud'].includes(deployment.profile)) {
    throw new Error('deployment.profile must be standalone or cloud');
  }
  const applicationRoot = resolveConfigRelativePath(
    configDirectory,
    deployment.applicationRoot,
    'deployment.applicationRoot',
  );
  if (!existsSync(path.join(applicationRoot, 'package.json'))) {
    throw new Error(`deployment.applicationRoot does not contain package.json: ${applicationRoot}`);
  }
  if (typeof deployment.serverScript !== 'string' || !deployment.serverScript.trim()) {
    throw new Error('deployment.serverScript must be a non-empty package script name');
  }
  const gateway = assertObject(deployment.gateway, 'deployment.gateway');
  assertExactKeys(
    gateway,
    ['host', 'port', 'httpPathPrefixes', 'webSocketPathPrefixes'],
    'deployment.gateway',
  );
  const applications = assertObject(config.applications, 'applications');
  assertExactKeys(applications, ['pc', 'h5'], 'applications');

  return {
    configPath: resolvedConfigPath,
    application: {
      manifestPath,
      deploymentConfigPath,
      environment: application.environment,
      protocol: publicUrl.protocol.slice(0, -1),
      publicHost: publicUrl.hostname.toLowerCase(),
      publicPort,
      publicUrl: publicUrl.toString(),
    },
    listener: {
      bind: validateHost(listener.bind, 'listener.bind'),
      certificate,
    },
    route: {
      pathPrefix: validatePathPrefix(route.pathPrefix),
      mobileUserAgentTokens: validateMobileTokens(route.mobileUserAgentTokens),
    },
    deployment: {
      profile: deployment.profile,
      applicationRoot,
      serverScript: deployment.serverScript,
      gateway: {
        host: validateHost(gateway.host, 'deployment.gateway.host'),
        port: validatePort(gateway.port, 'deployment.gateway.port'),
        httpPathPrefixes: validatePathPrefixes(
          gateway.httpPathPrefixes,
          'deployment.gateway.httpPathPrefixes',
        ),
        webSocketPathPrefixes: validatePathPrefixes(
          gateway.webSocketPathPrefixes,
          'deployment.gateway.webSocketPathPrefixes',
        ),
      },
    },
    applications: {
      pc: validateApplication(configDirectory, applications.pc, 'applications.pc'),
      h5: validateApplication(configDirectory, applications.h5, 'applications.h5'),
    },
  };
}
