const { app } = require('electron');
const { autoUpdater } = require('electron-updater');

function createKidOSUpdater(onStateChange = () => {}) {
  let state = {
    status: app.isPackaged ? 'idle' : 'development',
    currentVersion: app.getVersion(),
    availableVersion: null,
    percent: null,
    error: null,
  };

  function publish(patch) {
    state = { ...state, ...patch };
    onStateChange(state);
  }

  autoUpdater.autoDownload = false;
  autoUpdater.autoInstallOnAppQuit = false;
  autoUpdater.allowPrerelease = true;

  autoUpdater.on('checking-for-update', () => publish({ status: 'checking', error: null }));
  autoUpdater.on('update-available', (info) => publish({
    status: 'available',
    availableVersion: info.version || null,
    error: null,
  }));
  autoUpdater.on('update-not-available', () => publish({
    status: 'current',
    availableVersion: null,
    percent: null,
    error: null,
  }));
  autoUpdater.on('download-progress', (progress) => publish({
    status: 'downloading',
    percent: Math.max(0, Math.min(100, Math.round(progress.percent || 0))),
    error: null,
  }));
  autoUpdater.on('update-downloaded', (info) => publish({
    status: 'ready',
    availableVersion: info.version || state.availableVersion,
    percent: 100,
    error: null,
  }));
  autoUpdater.on('error', (error) => publish({
    status: 'error',
    error: error instanceof Error ? error.message : String(error),
  }));

  return {
    getState() {
      return state;
    },
    async check() {
      if (!app.isPackaged) {
        publish({ status: 'development', error: null });
        return state;
      }
      await autoUpdater.checkForUpdates();
      return state;
    },
    async download() {
      if (!app.isPackaged) throw new Error('Updates are available only in packaged KidOS builds.');
      if (state.status !== 'available' && state.status !== 'error') {
        throw new Error('No KidOS update is ready to download.');
      }
      publish({ status: 'downloading', percent: 0, error: null });
      await autoUpdater.downloadUpdate();
      return state;
    },
    install() {
      if (!app.isPackaged) throw new Error('Updates are available only in packaged KidOS builds.');
      if (state.status !== 'ready') throw new Error('The KidOS update has not finished downloading.');
      autoUpdater.quitAndInstall(false, true);
    },
  };
}

module.exports = { createKidOSUpdater };
