const { app, BrowserWindow, ipcMain, shell, session } = require('electron');
const path = require('node:path');

const isDev = Boolean(process.env.KIDOS_DESKTOP_DEV_URL);

function isTrustedNavigation(url) {
  if (isDev) return url.startsWith(process.env.KIDOS_DESKTOP_DEV_URL);
  return url.startsWith('file://');
}

function createWindow() {
  const win = new BrowserWindow({
    width: 1440,
    height: 900,
    minWidth: 1024,
    minHeight: 700,
    backgroundColor: '#0b1020',
    autoHideMenuBar: true,
    title: 'KidOS Desktop',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  });

  win.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('https://')) shell.openExternal(url);
    return { action: 'deny' };
  });

  win.webContents.on('will-navigate', (event, url) => {
    if (!isTrustedNavigation(url)) event.preventDefault();
  });

  if (isDev) {
    win.loadURL(process.env.KIDOS_DESKTOP_DEV_URL);
  } else {
    win.loadFile(path.join(process.resourcesPath, 'shell', 'index.html'));
  }

  return win;
}

app.whenReady().then(() => {
  session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => {
    callback(false);
  });

  ipcMain.handle('kidos-desktop:runtime', () => ({
    edition: 'desktop',
    guardianEnforcement: false,
    securityMode: 'desktop-preview',
    platform: process.platform,
  }));

  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
