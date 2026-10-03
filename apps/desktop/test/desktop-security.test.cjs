const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..');
const main = fs.readFileSync(path.join(root, 'electron', 'main.cjs'), 'utf8');
const preload = fs.readFileSync(path.join(root, 'electron', 'preload.cjs'), 'utf8');
const guardianClient = fs.readFileSync(path.join(root, 'electron', 'guardian-client.cjs'), 'utf8');
const installer = fs.readFileSync(path.join(root, 'build', 'installer.nsh'), 'utf8');
const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const viteConfig = fs.readFileSync(path.join(root, '..', 'shell', 'vite.config.ts'), 'utf8');
const shellApi = fs.readFileSync(path.join(root, '..', 'shell', 'src', 'lib', 'kidos-api.ts'), 'utf8');

test('Electron renderer is isolated from Node', () => {
  assert.match(main, /contextIsolation:\s*true/);
  assert.match(main, /nodeIntegration:\s*false/);
  assert.match(main, /sandbox:\s*true/);
});

test('Electron runtime uses the Guardian security mode instead of preview claims', () => {
  assert.match(main, /securityMode:\s*'guardian'/);
  assert.match(main, /restricted_safe_mode/);
  assert.match(main, /guardianStatus\(\)/);
});

test('Guardian client is bounded to the KidOS named pipe and protocol', () => {
  assert.match(guardianClient, /KidOSGuardian\.v1/);
  assert.match(guardianClient, /MAX_MESSAGE_BYTES\s*=\s*64 \* 1024/);
  assert.match(guardianClient, /randomUUID/);
  assert.doesNotMatch(guardianClient, /child_process|exec\(|spawn\(/);
});

test('preload exposes only explicit KidOS methods', () => {
  assert.match(preload, /contextBridge\.exposeInMainWorld\('kidosDesktop'/);
  assert.match(preload, /guardianStatus/);
  assert.match(preload, /configureWindowsLockdown/);
  assert.match(preload, /listApprovedApps/);
  assert.match(preload, /runParentRecovery/);
  assert.doesNotMatch(preload, /require:\s*require|process:\s*process/);
  assert.doesNotMatch(preload, /ipcRenderer\.send\([^'"]/);
});

test('Windows packaging is an elevated per-machine KidOS NSIS installer', () => {
  assert.equal(pkg.build.productName, 'KidOS');
  assert.equal(pkg.build.win.target[0].target, 'nsis');
  assert.deepEqual(pkg.build.win.target[0].arch, ['x64']);
  assert.equal(pkg.build.win.requestedExecutionLevel, 'asInvoker');
  assert.equal(pkg.build.nsis.perMachine, true);
  assert.equal(pkg.build.nsis.oneClick, false);
  assert.equal(pkg.build.nsis.include, 'build/installer.nsh');
});

test('full Electron package carries Guardian and local classifier resources', () => {
  const resources = JSON.stringify(pkg.build.extraResources);
  assert.match(resources, /kidos-guardian-host\.exe/);
  assert.match(resources, /kidos-media-classifier-bundle\.zip/);
  assert.match(resources, /services\/media-classifier\/dist\/model|services\\media-classifier\\dist\\model|dist\/model/);
  assert.match(installer, /install-electron-backend\.ps1/);
  assert.match(installer, /uninstall-electron-backend\.ps1/);
});

test('production shell uses relative assets for Electron file loading', () => {
  assert.match(viteConfig, /base:\s*['"]\.\/['"]/);
});

test('Electron shell uses the real backend runtime adapter', () => {
  assert.match(shellApi, /electronKidOSApi/);
  assert.match(shellApi, /getParentPolicy/);
  assert.match(shellApi, /listApprovedApps/);
  assert.match(shellApi, /askAi/);
  assert.doesNotMatch(shellApi, /desktopPreviewKidOSApi/);
});

test('packaged Electron build includes a renderer smoke-test gate', () => {
  assert.match(main, /KIDOS_ELECTRON_SMOKE/);
  assert.match(main, /data-testid="kidos-shell"/);
  assert.match(main, /did-fail-load/);
});

test('approved online resources open through the protected browser', () => {
  assert.match(main, /APPROVED_ONLINE_RESOURCES/);
  assert.match(main, /openProtectedBrowser\(url\)/);
  assert.doesNotMatch(main, /shell\.openExternal\(url\)/);
});

test('approved app launch resolves the path from Guardian, not renderer input', () => {
  assert.match(main, /listApprovedAppsInternal/);
  assert.match(main, /launch-approved-app/);
  assert.match(main, /selected\.executable_path/);
  assert.doesNotMatch(preload, /executablePath/);
});
