import assert from 'node:assert/strict';
import test from 'node:test';

import {
  isMobileUserAgent,
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
  assert.equal(config.route.pathPrefix, '/sdkwork-im/');
  assert.equal(config.https.port, 3443);
  assert.equal(config.https.certificate.mode, 'auto');
  assert.equal(config.applications.pc.port, 4176);
  assert.equal(config.applications.h5.port, 4177);
  assert.match(config.applications.pc.root, /sdkwork-im[\\/]apps[\\/]sdkwork-im-pc$/u);
  assert.match(config.applications.h5.root, /sdkwork-im[\\/]apps[\\/]sdkwork-im-h5$/u);
});
