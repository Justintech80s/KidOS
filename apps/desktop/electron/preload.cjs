const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('kidosDesktop', Object.freeze({
  runtime: () => ipcRenderer.invoke('kidos-desktop:runtime'),
  openApprovedResource: (resource) => ipcRenderer.invoke('kidos-desktop:open-approved-resource', resource),
  edition: 'desktop',
}));
