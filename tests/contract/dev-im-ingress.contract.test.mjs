import assert from 'node:assert/strict';
import test from 'node:test';

import {
  isMobileUserAgent,
  requestPathMatchesPrefix,
  requestHostMatches,
  selectImDevTarget,
} from '../../scripts/dev-im-ingress.mjs';
import {
  DEFAULT_IM_DEV_CONFIG_PATH,
  loadImDevConfig,
} from '../../scripts/lib/im-dev-config.mjs';

const config = loadImDevConfig(DEFAULT_IM_DEV_CONFIG_PATH);
const settings = {
  mobileUserAgentTokens: config.route.mobileUserAgentTokens,
  targets: config.applications,
};

test('mobile browsers select sdkwork-im-h5', () => {
  const iphone = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) Mobile/15E148';
  const android = 'Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 Mobile Safari/537.36';

  assert.equal(isMobileUserAgent(iphone, settings.mobileUserAgentTokens), true);
  assert.equal(selectImDevTarget(iphone, settings), settings.targets.h5);
  assert.equal(selectImDevTarget(android, settings), settings.targets.h5);
});

test('desktop browsers and missing user agents select sdkwork-im-pc', () => {
  const desktop = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/136.0 Safari/537.36';

  assert.equal(isMobileUserAgent(desktop, settings.mobileUserAgentTokens), false);
  assert.equal(selectImDevTarget(desktop, settings), settings.targets.pc);
  assert.equal(selectImDevTarget(undefined, settings), settings.targets.pc);
});

test('tracked etc config owns the development ingress topology', () => {
  assert.equal(config.application.environment, 'development');
  assert.equal(config.application.protocol, 'http');
  assert.equal(config.application.publicHost, 'im-dev.sdkwork.com');
  assert.equal(config.application.publicPort, 3801);
  assert.equal(config.application.publicUrl, 'http://im-dev.sdkwork.com:3801/');
  assert.match(
    config.application.deploymentConfigPath,
    /sdkwork-im[\\/]etc[\\/]sdkwork\.deployment\.config\.json$/u,
  );
  assert.equal(config.listener.bind, '0.0.0.0');
  assert.equal(config.route.pathPrefix, '/');
  assert.equal(config.deployment.profile, 'standalone');
  assert.equal(config.deployment.gateway.host, '127.0.0.1');
  assert.equal(config.deployment.gateway.port, 18079);
  assert.match(config.deployment.applicationRoot, /sdkwork-im$/u);
  assert.equal(config.deployment.serverScript, 'dev:server');
  assert.equal(config.applications.pc.port, 4176);
  assert.equal(config.applications.h5.port, 4177);
  assert.match(config.applications.pc.root, /sdkwork-im[\\/]apps[\\/]sdkwork-im-pc$/u);
  assert.match(config.applications.h5.root, /sdkwork-im[\\/]apps[\\/]sdkwork-im-h5$/u);
});

test('standalone API and realtime routes take precedence over renderer fallback', () => {
  const { httpPathPrefixes, webSocketPathPrefixes } = config.deployment.gateway;

  assert.equal(requestPathMatchesPrefix('/im/v3/api/conversations?page=1', httpPathPrefixes), true);
  assert.equal(requestPathMatchesPrefix('/app/v3/api/auth/sessions', httpPathPrefixes), true);
  assert.equal(requestPathMatchesPrefix('/backend/v3/api/runtime', httpPathPrefixes), true);
  assert.equal(requestPathMatchesPrefix('/healthz', httpPathPrefixes), true);
  assert.equal(requestPathMatchesPrefix('/@vite/client', httpPathPrefixes), false);
  assert.equal(requestPathMatchesPrefix('/conversations/123', httpPathPrefixes), false);
  assert.equal(
    requestPathMatchesPrefix('/im/v3/api/realtime/ws?deviceId=test', webSocketPathPrefixes),
    true,
  );
  assert.equal(requestPathMatchesPrefix('/', webSocketPathPrefixes), false);
});

test('ingress requires the exact configured application host and port', () => {
  assert.equal(requestHostMatches('im-dev.sdkwork.com:3801', 'im-dev.sdkwork.com', 3801), true);
  assert.equal(requestHostMatches('IM-DEV.SDKWORK.COM:3801', 'im-dev.sdkwork.com', 3801), true);
  assert.equal(requestHostMatches('localhost:3801', 'im-dev.sdkwork.com', 3801), false);
  assert.equal(requestHostMatches('im-dev.sdkwork.com:3802', 'im-dev.sdkwork.com', 3801), false);
  assert.equal(requestHostMatches('im-dev.sdkwork.com', 'im-dev.sdkwork.com', 3801), false);
  assert.equal(requestHostMatches('im.sdkwork.com', 'im.sdkwork.com', 443), true);
});
