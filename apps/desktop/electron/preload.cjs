const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('kidosDesktop', Object.freeze({
  runtime: () => ipcRenderer.invoke('kidos-desktop:runtime'),
  edition: 'desktop',
}));
