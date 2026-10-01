const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..');
const main = fs.readFileSync(path.join(root, 'electron', 'main.cjs'), 'utf8');
const preload = fs.readFileSync(path.join(root, 'electron', 'preload.cjs'), 'utf8');
const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const viteConfig = fs.readFileSync(path.join(root, '..', 'shell', 'vite.config.ts'), 'utf8');
const shellApi = fs.readFileSync(path.join(root, '..', 'shell', 'src', 'lib', 'kidos-api.ts'), 'utf8');

test('Electron renderer is isolated from Node', () => {
  assert.match(main, /contextIsolation:\s*true/);
  assert.match(main, /nodeIntegration:\s*false/);
  assert.match(main, /sandbox:\s*true/);
});

test('desktop edition does not pretend Guardian enforcement is active', () => {
  assert.match(main, /guardianEnforcement:\s*false/);
  assert.match(main, /securityMode:\s*'desktop-preview'/);
});

test('preload exposes only the bounded KidOS Desktop bridge', () => {
  assert.match(preload, /contextBridge\.exposeInMainWorld\('kidosDesktop'/);
  assert.doesNotMatch(preload, /require:\s*require|process:\s*process/);
});

test('Windows packaging is NSIS x64', () => {
  assert.equal(pkg.build.productName, 'KidOS Desktop');
  assert.equal(pkg.build.win.target[0].target, 'nsis');
  assert.deepEqual(pkg.build.win.target[0].arch, ['x64']);
});


test('production shell uses relative assets for Electron file loading', () => {
  assert.match(viteConfig, /base:\s*['"]\.\/['"]/);
});

test('Electron desktop preview has a non-Tauri runtime adapter', () => {
  assert.match(shellApi, /desktopPreviewKidOSApi/);
  assert.match(shellApi, /isDesktopPreviewRuntime/);
  assert.match(shellApi, /evaluateNavigation\(\) \{ return Promise\.resolve<PolicyDecision>\('block'\); \}/);
});


test('packaged Electron build includes a renderer smoke-test gate', () => {
  assert.match(main, /KIDOS_ELECTRON_SMOKE/);
  assert.match(main, /data-testid="kidos-shell"/);
  assert.match(main, /did-fail-load/);
});
