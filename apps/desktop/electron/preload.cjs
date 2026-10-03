const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('kidosDesktop', Object.freeze({
  edition: 'desktop',
  runtime: () => ipcRenderer.invoke('kidos-desktop:runtime'),
  guardianStatus: () => ipcRenderer.invoke('kidos-desktop:guardian-status'),
  planWorkspace: (prompt) => ipcRenderer.invoke('kidos-desktop:plan-workspace', prompt),
  evaluateNavigation: (url) => ipcRenderer.invoke('kidos-desktop:evaluate-navigation', url),
  evaluateDownload: (fileName, mimeType) => ipcRenderer.invoke('kidos-desktop:evaluate-download', fileName, mimeType),
  openProtectedBrowser: (url) => ipcRenderer.invoke('kidos-desktop:open-protected-browser', url),

  parentSetupStatus: () => ipcRenderer.invoke('kidos-desktop:parent-setup-status'),
  configureParentPin: (pin, currentPin) => ipcRenderer.invoke('kidos-desktop:configure-parent-pin', pin, currentPin),
  verifyParentPin: (pin) => ipcRenderer.invoke('kidos-desktop:verify-parent-pin', pin),
  getParentPolicy: () => ipcRenderer.invoke('kidos-desktop:get-parent-policy'),
  saveParentPolicy: (pin, policy) => ipcRenderer.invoke('kidos-desktop:save-parent-policy', pin, policy),

  lockdownStatus: () => ipcRenderer.invoke('kidos-desktop:lockdown-status'),
  configureWindowsLockdown: (request) => ipcRenderer.invoke('kidos-desktop:configure-lockdown', request),
  requestParentMaintenanceUnlock: (pin, durationMinutes) => ipcRenderer.invoke('kidos-desktop:parent-unlock', pin, durationMinutes),
  removeWindowsLockdown: (pin) => ipcRenderer.invoke('kidos-desktop:remove-lockdown', pin),

  listApprovedApps: () => ipcRenderer.invoke('kidos-desktop:list-approved-apps'),
  launchApprovedApp: (appId) => ipcRenderer.invoke('kidos-desktop:launch-approved-app', appId),

  listQuarantineMedia: (pin) => ipcRenderer.invoke('kidos-desktop:list-quarantine', pin),
  previewQuarantineMedia: (pin, itemId) => ipcRenderer.invoke('kidos-desktop:preview-quarantine', pin, itemId),
  reviewQuarantineMedia: (pin, itemId, action) => ipcRenderer.invoke('kidos-desktop:review-quarantine', pin, itemId, action),
  getRecoveryStatus: () => ipcRenderer.invoke('kidos-desktop:recovery-status'),
  runParentRecovery: (pin, action) => ipcRenderer.invoke('kidos-desktop:run-recovery', pin, action),

  askAi: (query) => ipcRenderer.invoke('kidos-desktop:ask-ai', query),
  getWellbeing: () => ipcRenderer.invoke('kidos-desktop:get-wellbeing'),
  saveWellbeing: (value) => ipcRenderer.invoke('kidos-desktop:save-wellbeing', value),

  openApprovedResource: (resource) => ipcRenderer.invoke('kidos-desktop:open-approved-resource', resource),
}));
