const { app, BrowserWindow, ipcMain, shell, session } = require('electron');
const fs = require('node:fs/promises');
const path = require('node:path');
const { requestGuardian } = require('./guardian-client.cjs');

const isDev = Boolean(process.env.KIDOS_DESKTOP_DEV_URL);
const isSmokeTest = process.env.KIDOS_ELECTRON_SMOKE === '1';
const LOCKDOWN_PROFILE_ID = '{7B62A1F3-8B61-4E6F-9E13-7A4C4A53E9D1}';

const APPROVED_ONLINE_RESOURCES = Object.freeze({
  kiddle: 'https://www.kiddle.co/',
  'khan-kids': 'https://www.khanacademy.org/kids',
  'pbs-kids': 'https://pbskids.org/games/',
  'youtube-kids': 'https://kids.youtube.com/',
  'family-safety': 'https://account.microsoft.com/family',
  khanmigo: 'https://www.khanacademy.org/khan-labs',
});

function isTrustedNavigation(url) {
  if (isDev) return url.startsWith(process.env.KIDOS_DESKTOP_DEV_URL);
  return url.startsWith('file://');
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

async function guardian(type, fields = {}) {
  return requestGuardian({ type, ...fields });
}

async function guardianRecoveryStatus() {
  const response = await guardian('recovery_status');
  if (response.type !== 'recovery_status') throw new Error('Guardian returned an unexpected recovery response.');
  return response;
}

async function guardianStatus() {
  const status = await guardianRecoveryStatus();
  return status.guardian_healthy && status.policy_valid ? 'healthy' : 'restricted_safe_mode';
}

async function getParentPolicy() {
  const response = await guardian('get_parent_policy');
  if (response.type !== 'parent_policy') throw new Error('Guardian returned an unexpected policy response.');
  return response.policy;
}

function workspacePlan(prompt) {
  const normalized = String(prompt || '').toLowerCase();
  if (['code', 'coding', 'game', 'program'].some((term) => normalized.includes(term))) {
    return { kind: 'beginner_coding', title: 'Beginner Coding', capabilities: ['beginner_coding', 'export_project'] };
  }
  if (['draw', 'picture', 'poster', 'presentation', 'slides', 'cartoon'].some((term) => normalized.includes(term))) {
    return { kind: 'drawing_presentation', title: 'Draw & Present', capabilities: ['drawing_presentation', 'export_project'] };
  }
  return {
    kind: 'story',
    title: normalized.includes('space') ? 'Space Story' : 'Story',
    capabilities: ['story', 'export_project'],
  };
}

async function saveWorkspacePlan(prompt) {
  const plan = workspacePlan(prompt);
  const directory = path.join(app.getPath('userData'), 'workspaces');
  await fs.mkdir(directory, { recursive: true });
  const stamp = new Date().toISOString().replace(/[:.]/g, '-');
  const record = { ...plan, prompt: String(prompt).slice(0, 2000), createdAt: new Date().toISOString() };
  await fs.writeFile(path.join(directory, `${stamp}.json`), JSON.stringify(record, null, 2), { encoding: 'utf8', mode: 0o600 });
  return plan;
}

function hardenWebUrl(candidate) {
  const url = new URL(candidate);
  if (!['http:', 'https:'].includes(url.protocol)) throw new Error('KidOS Safe Browser only allows web URLs.');
  const host = url.hostname.toLowerCase();
  if ((host === 'google.com' || host.endsWith('.google.com')) && url.pathname.startsWith('/search')) {
    url.searchParams.set('safe', 'active');
  }
  if ((host === 'bing.com' || host.endsWith('.bing.com')) && url.pathname.startsWith('/search')) {
    url.searchParams.set('adlt', 'strict');
  }
  return url.toString();
}

async function navigationDecision(candidate) {
  let url;
  try {
    url = hardenWebUrl(candidate);
  } catch {
    return { decision: 'block', url: candidate };
  }
  const response = await guardian('evaluate_navigation', { url });
  if (response.type !== 'policy_decision') throw new Error('Guardian returned an unexpected navigation response.');
  return { decision: response.decision, url };
}

let protectedBrowserSession;
function getProtectedBrowserSession() {
  if (protectedBrowserSession) return protectedBrowserSession;
  protectedBrowserSession = session.fromPartition('persist:kidos-safe-browser', { cache: true });
  protectedBrowserSession.setPermissionRequestHandler((_webContents, _permission, callback) => callback(false));
  protectedBrowserSession.setPermissionCheckHandler(() => false);
  protectedBrowserSession.on('will-download', (event, item) => {
    event.preventDefault();
    console.warn('KidOS Safe Browser blocked an unmanaged download.', { url: item.getURL(), fileName: item.getFilename() });
  });
  return protectedBrowserSession;
}

async function openProtectedBrowser(candidate) {
  const first = await navigationDecision(candidate);
  if (first.decision !== 'allow') {
    throw new Error(first.decision === 'require_parent' ? 'Parent approval is required for this destination.' : 'KidOS Guardian blocked this destination.');
  }

  const browser = new BrowserWindow({
    title: 'KidOS Safe Browser',
    width: 1280,
    height: 860,
    autoHideMenuBar: true,
    backgroundColor: '#0b1020',
    webPreferences: {
      partition: 'persist:kidos-safe-browser',
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  });

  let approvedOnce = null;
  const navigate = async (next) => {
    try {
      const checked = await navigationDecision(next);
      if (checked.decision !== 'allow') return;
      approvedOnce = checked.url;
      await browser.loadURL(checked.url);
    } catch (error) {
      console.warn('KidOS Safe Browser navigation denied.', errorMessage(error));
    }
  };

  browser.webContents.setWindowOpenHandler(({ url }) => {
    void navigate(url);
    return { action: 'deny' };
  });

  browser.webContents.on('will-navigate', (event, url) => {
    if (approvedOnce === url) {
      approvedOnce = null;
      return;
    }
    event.preventDefault();
    void navigate(url);
  });

  browser.webContents.on('will-redirect', (event, url) => {
    if (approvedOnce === url) {
      approvedOnce = null;
      return;
    }
    event.preventDefault();
    void navigate(url);
  });

  approvedOnce = first.url;
  await browser.loadURL(first.url);
  return true;
}

function lockdownStatusDto(response, managedAccount) {
  return {
    state: response.state,
    capability: { platform: 'windows', supported: true, mechanism: 'assigned_access' },
    ...(managedAccount ? { managedAccount } : {}),
    ...(response.reason ? { reason: response.reason } : {}),
  };
}

async function listApprovedAppsInternal() {
  const response = await guardian('list_approved_apps');
  if (response.type !== 'approved_apps') throw new Error('Guardian returned an unexpected approved-app response.');
  return Array.isArray(response.apps) ? response.apps : [];
}

async function askKidOSAi(query) {
  const text = String(query || '').trim();
  if (!text) throw new Error('Ask KidOS AI a question first.');
  const endpoint = process.env.KIDOS_AI_ENDPOINT;
  if (!endpoint) {
    return {
      available: false,
      answer: 'KidOS AI is not connected to a production AI service on this computer yet. Learning, browsing, Guardian, and parent controls remain protected.',
    };
  }
  const policy = await getParentPolicy();
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(process.env.KIDOS_AI_TOKEN ? { authorization: `Bearer ${process.env.KIDOS_AI_TOKEN}` } : {}),
    },
    body: JSON.stringify({
      query: text.slice(0, 4000),
      childAge: policy.childAge,
      safetyMode: 'child',
      instruction: 'Answer age-appropriately. Do not provide unsafe, sexual, violent, self-harm, drug, weapon, evasion, or privacy-invasive instructions. Encourage a parent or teacher when appropriate.',
    }),
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) throw new Error(`KidOS AI service returned HTTP ${response.status}.`);
  const data = await response.json();
  if (!data || typeof data.answer !== 'string' || !data.answer.trim()) throw new Error('KidOS AI returned an invalid response.');
  return { available: true, answer: data.answer.trim().slice(0, 12000) };
}

async function loadWellbeing() {
  const file = path.join(app.getPath('userData'), 'wellbeing.json');
  try {
    return JSON.parse(await fs.readFile(file, 'utf8'));
  } catch {
    return { dailyMinutes: 120, breakEveryMinutes: 30, windDownHour: 20, largeText: false, reducedMotion: false };
  }
}

async function saveWellbeing(value) {
  const next = {
    dailyMinutes: Math.min(600, Math.max(15, Number(value?.dailyMinutes) || 120)),
    breakEveryMinutes: Math.min(120, Math.max(10, Number(value?.breakEveryMinutes) || 30)),
    windDownHour: Math.min(23, Math.max(0, Number(value?.windDownHour) || 20)),
    largeText: Boolean(value?.largeText),
    reducedMotion: Boolean(value?.reducedMotion),
  };
  await fs.writeFile(path.join(app.getPath('userData'), 'wellbeing.json'), JSON.stringify(next, null, 2), { encoding: 'utf8', mode: 0o600 });
  return next;
}

function createWindow() {
  const win = new BrowserWindow({
    width: 1440,
    height: 900,
    minWidth: 1024,
    minHeight: 700,
    backgroundColor: '#0b1020',
    autoHideMenuBar: true,
    title: 'KidOS',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  });

  win.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));

  win.webContents.on('will-navigate', (event, url) => {
    if (!isTrustedNavigation(url)) event.preventDefault();
  });

  win.webContents.on('did-fail-load', (_event, errorCode, errorDescription, validatedURL) => {
    console.error('KidOS failed to load', { errorCode, errorDescription, validatedURL });
    if (isSmokeTest) app.exit(1);
  });

  if (isSmokeTest) {
    win.webContents.on('did-finish-load', async () => {
      try {
        const result = await win.webContents.executeJavaScript(`
          new Promise((resolve) => {
            const deadline = Date.now() + 10000;
            const check = () => {
              const shell = document.querySelector('[data-testid="kidos-shell"]');
              if (shell) return resolve({ ok: true, text: document.body.innerText.slice(0, 500) });
              if (Date.now() >= deadline) return resolve({ ok: false, text: document.body.innerText.slice(0, 500) });
              setTimeout(check, 100);
            };
            check();
          })
        `);
        console.log('KidOS Electron renderer smoke result', result);
        app.exit(result.ok ? 0 : 1);
      } catch (error) {
        console.error('KidOS Electron renderer smoke failed', error);
        app.exit(1);
      }
    });
  }

  if (isDev) win.loadURL(process.env.KIDOS_DESKTOP_DEV_URL);
  else win.loadFile(path.join(process.resourcesPath, 'shell', 'index.html'));

  return win;
}

app.whenReady().then(() => {
  session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => callback(false));

  ipcMain.handle('kidos-desktop:runtime', async () => {
    try {
      return { edition: 'desktop', guardianEnforcement: (await guardianStatus()) === 'healthy', securityMode: 'guardian', platform: process.platform };
    } catch {
      return { edition: 'desktop', guardianEnforcement: false, securityMode: 'restricted_safe_mode', platform: process.platform };
    }
  });

  ipcMain.handle('kidos-desktop:guardian-status', () => guardianStatus());
  ipcMain.handle('kidos-desktop:plan-workspace', (_event, prompt) => saveWorkspacePlan(prompt));
  ipcMain.handle('kidos-desktop:evaluate-navigation', async (_event, url) => (await navigationDecision(url)).decision);
  ipcMain.handle('kidos-desktop:evaluate-download', async (_event, fileName, mimeType) => {
    const response = await guardian('evaluate_download', {
      url: 'https://kidos.local/renderer-request',
      file_name: String(fileName || '').slice(0, 260),
      mime_type: String(mimeType || 'application/octet-stream').slice(0, 200),
      archive_contains_high_risk: false,
    });
    if (response.type !== 'policy_decision') throw new Error('Guardian returned an unexpected download response.');
    return response.decision;
  });
  ipcMain.handle('kidos-desktop:open-protected-browser', (_event, url) => openProtectedBrowser(url));

  ipcMain.handle('kidos-desktop:parent-setup-status', async () => {
    const response = await guardian('parent_setup_status');
    if (response.type !== 'parent_setup') throw new Error('Guardian returned an unexpected parent setup response.');
    return Boolean(response.configured);
  });
  ipcMain.handle('kidos-desktop:configure-parent-pin', async (_event, pin, currentPin) => {
    const response = await guardian('configure_parent_pin', { new_pin: String(pin), current_pin: currentPin ? String(currentPin) : null });
    if (response.type !== 'ack') throw new Error('Guardian did not confirm parent PIN configuration.');
  });
  ipcMain.handle('kidos-desktop:verify-parent-pin', async (_event, pin) => {
    const response = await guardian('verify_parent_pin', { pin: String(pin) });
    if (response.type !== 'parent_verification') throw new Error('Guardian returned an unexpected PIN response.');
    return { authorized: Boolean(response.authorized), locked: Boolean(response.locked) };
  });
  ipcMain.handle('kidos-desktop:get-parent-policy', () => getParentPolicy());
  ipcMain.handle('kidos-desktop:save-parent-policy', async (_event, pin, policy) => {
    const response = await guardian('save_parent_policy', { pin: String(pin), policy });
    if (response.type !== 'ack') throw new Error('Guardian did not confirm the parent policy.');
    return { saved: true };
  });

  ipcMain.handle('kidos-desktop:lockdown-status', async () => lockdownStatusDto(await guardian('status')));
  ipcMain.handle('kidos-desktop:configure-lockdown', async (_event, request) => {
    if (!request?.parentPin) throw new Error('Parent PIN is required for Windows lockdown.');
    const account = request.account;
    const approved = Array.isArray(request.approvedApps) ? request.approvedApps : [];
    const apps = approved.map((entry) => ({
      id: String(entry.id),
      display_name: String(entry.displayName),
      executable_path: entry.id === 'kidos' ? process.execPath : String(entry.executablePath),
    }));
    if (!apps.some((entry) => entry.id.toLowerCase() === 'kidos')) {
      apps.unshift({ id: 'kidos', display_name: 'KidOS', executable_path: process.execPath });
    }
    const response = await guardian('apply_lockdown', {
      pin: String(request.parentPin),
      profile: {
        profile_id: LOCKDOWN_PROFILE_ID,
        account: String(account?.id || ''),
        account_role: String(account?.role || 'unknown'),
        apps,
      },
    });
    if (response.type !== 'status') throw new Error('Guardian returned an unexpected lockdown response.');
    return lockdownStatusDto(response, account);
  });
  ipcMain.handle('kidos-desktop:parent-unlock', async (_event, pin, durationMinutes) => {
    const minutes = Math.min(60, Math.max(1, Number(durationMinutes) || 15));
    const response = await guardian('parent_unlock', { pin: String(pin), duration_minutes: minutes });
    if (response.type !== 'status') throw new Error('Guardian returned an unexpected unlock response.');
    const now = Date.now();
    return { grantedAt: new Date(now).toISOString(), expiresAt: new Date(now + minutes * 60000).toISOString() };
  });
  ipcMain.handle('kidos-desktop:remove-lockdown', async (_event, pin) => {
    const response = await guardian('remove_lockdown', { pin: String(pin) });
    if (response.type !== 'status') throw new Error('Guardian returned an unexpected lockdown response.');
    return lockdownStatusDto(response);
  });

  ipcMain.handle('kidos-desktop:list-approved-apps', async () => {
    const apps = await listApprovedAppsInternal();
    return apps.filter((entry) => String(entry.id).toLowerCase() !== 'kidos').map((entry) => ({ id: entry.id, displayName: entry.display_name }));
  });
  ipcMain.handle('kidos-desktop:launch-approved-app', async (_event, appId) => {
    const apps = await listApprovedAppsInternal();
    const selected = apps.find((entry) => entry.id === appId && String(entry.id).toLowerCase() !== 'kidos');
    if (!selected) throw new Error('This app is not in the active Guardian approved-app profile.');
    const result = await shell.openPath(selected.executable_path);
    if (result) throw new Error(result);
    return true;
  });

  ipcMain.handle('kidos-desktop:list-quarantine', async (_event, pin) => {
    const response = await guardian('list_quarantine', { pin: String(pin) }, 10000);
    if (response.type !== 'quarantine_items') throw new Error('Guardian returned an unexpected quarantine response.');
    return response.items.map((item) => ({
      id: item.id, fileName: item.file_name, sizeBytes: item.size_bytes, modifiedSeconds: item.modified_seconds,
      category: item.category ?? undefined, risk: item.risk ?? undefined, confidence: item.confidence ?? undefined, reason: item.reason ?? undefined,
    }));
  });
  ipcMain.handle('kidos-desktop:preview-quarantine', async (_event, pin, itemId) => {
    const response = await guardian('preview_quarantine', { pin: String(pin), item_id: String(itemId) }, 25000);
    if (response.type !== 'quarantine_preview') throw new Error('Guardian returned an unexpected preview response.');
    return { mimeType: response.mime_type, dataBase64: response.data_base64 };
  });
  ipcMain.handle('kidos-desktop:review-quarantine', async (_event, pin, itemId, action) => {
    const response = await guardian('review_quarantine', { pin: String(pin), item_id: String(itemId), action: String(action) });
    if (response.type !== 'ack') throw new Error('Guardian did not confirm the quarantine action.');
  });
  ipcMain.handle('kidos-desktop:recovery-status', async () => {
    const response = await guardianRecoveryStatus();
    return {
      guardianHealthy: Boolean(response.guardian_healthy),
      classifierHealthy: Boolean(response.classifier_healthy),
      recoveryRequired: Boolean(response.recovery_required),
      recoveryReason: response.recovery_reason ?? undefined,
      policyValid: Boolean(response.policy_valid),
      lockdownState: response.lockdown_state,
    };
  });
  ipcMain.handle('kidos-desktop:run-recovery', async (_event, pin, action) => {
    const response = await guardian('run_recovery', { pin: String(pin), action: String(action) });
    if (response.type !== 'ack') throw new Error('Guardian did not confirm recovery.');
    return response.message;
  });

  ipcMain.handle('kidos-desktop:ask-ai', (_event, query) => askKidOSAi(query));
  ipcMain.handle('kidos-desktop:get-wellbeing', () => loadWellbeing());
  ipcMain.handle('kidos-desktop:save-wellbeing', (_event, value) => saveWellbeing(value));

  ipcMain.handle('kidos-desktop:open-approved-resource', async (_event, resource) => {
    const url = APPROVED_ONLINE_RESOURCES[resource];
    if (!url) return false;
    await openProtectedBrowser(url);
    return true;
  });

  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
