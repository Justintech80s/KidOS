const fs = require('node:fs');
const fsp = require('node:fs/promises');
const path = require('node:path');
const { DatabaseSync } = require('node:sqlite');
const { randomUUID } = require('node:crypto');

const SCHEMA_VERSION = 1;
const DEFAULT_WELLBEING = Object.freeze({
  dailyMinutes: 120,
  breakEveryMinutes: 30,
  windDownHour: 20,
  largeText: false,
  reducedMotion: false,
});

function clampNumber(value, min, max, fallback) {
  const number = Number(value);
  return Number.isFinite(number) ? Math.min(max, Math.max(min, Math.round(number))) : fallback;
}

function normalizeWellbeing(value, previous = DEFAULT_WELLBEING, allowLimits = false) {
  return {
    dailyMinutes: allowLimits ? clampNumber(value?.dailyMinutes, 15, 600, previous.dailyMinutes) : previous.dailyMinutes,
    breakEveryMinutes: allowLimits ? clampNumber(value?.breakEveryMinutes, 10, 120, previous.breakEveryMinutes) : previous.breakEveryMinutes,
    windDownHour: allowLimits ? clampNumber(value?.windDownHour, 0, 23, previous.windDownHour) : previous.windDownHour,
    largeText: Boolean(value?.largeText),
    reducedMotion: Boolean(value?.reducedMotion),
  };
}

function normalizeWorkspaceDocument(input, existing) {
  const allowedKinds = new Set(['story', 'drawing_presentation', 'beginner_coding']);
  const kind = allowedKinds.has(String(input?.kind)) ? String(input.kind) : 'story';
  const id = /^[A-Za-z0-9-]{1,80}$/.test(String(input?.id || '')) ? String(input.id) : randomUUID();
  const now = new Date().toISOString();
  return {
    id,
    kind,
    title: String(input?.title || 'Untitled KidOS Project').trim().slice(0, 80) || 'Untitled KidOS Project',
    prompt: String(input?.prompt || '').slice(0, 2000),
    content: String(input?.content || '').slice(0, 100000),
    createdAt: existing?.createdAt || now,
    updatedAt: now,
  };
}

function sqlString(value) {
  return "'" + String(value).replaceAll("'", "''") + "'";
}

function openKidOSDataStore(userDataDir) {
  fs.mkdirSync(userDataDir, { recursive: true });
  const databasePath = path.join(userDataDir, 'kidos.sqlite3');
  const db = new DatabaseSync(databasePath);
  db.exec('PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;');

  db.exec(`
    CREATE TABLE IF NOT EXISTS meta (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS workspace_documents (
      id TEXT PRIMARY KEY,
      kind TEXT NOT NULL,
      title TEXT NOT NULL,
      prompt TEXT NOT NULL,
      content TEXT NOT NULL,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS wellbeing (
      id INTEGER PRIMARY KEY CHECK(id = 1),
      daily_minutes INTEGER NOT NULL,
      break_every_minutes INTEGER NOT NULL,
      wind_down_hour INTEGER NOT NULL,
      large_text INTEGER NOT NULL,
      reduced_motion INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS usage_daily (
      day TEXT PRIMARY KEY,
      active_seconds INTEGER NOT NULL DEFAULT 0
    );
  `);

  const version = Number(db.prepare("SELECT value FROM meta WHERE key='schema_version'").get()?.value || 0);
  if (version < SCHEMA_VERSION) {
    createBackup('pre-migration');
    db.prepare("INSERT INTO meta(key,value) VALUES('schema_version',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
      .run(String(SCHEMA_VERSION));
  }

  const hasWellbeing = db.prepare('SELECT 1 AS present FROM wellbeing WHERE id=1').get();
  if (!hasWellbeing) {
    db.prepare(`INSERT INTO wellbeing(id,daily_minutes,break_every_minutes,wind_down_hour,large_text,reduced_motion)
                VALUES(1,?,?,?,?,?)`).run(
      DEFAULT_WELLBEING.dailyMinutes,
      DEFAULT_WELLBEING.breakEveryMinutes,
      DEFAULT_WELLBEING.windDownHour,
      0,
      0,
    );
  }

  function createBackup(reason = 'manual') {
    const backupDir = path.join(userDataDir, 'backups');
    fs.mkdirSync(backupDir, { recursive: true });
    const stamp = new Date().toISOString().replace(/[:.]/g, '-');
    const backupPath = path.join(backupDir, `kidos-${reason}-${stamp}.sqlite3`);
    try {
      db.exec(`VACUUM INTO ${sqlString(backupPath)}`);
    } catch {
      // A brand-new database may not yet have meaningful content to back up.
      return null;
    }
    const backups = fs.readdirSync(backupDir)
      .filter((name) => /^kidos-.*\.sqlite3$/.test(name))
      .map((name) => ({ name, full: path.join(backupDir, name), time: fs.statSync(path.join(backupDir, name)).mtimeMs }))
      .sort((a, b) => b.time - a.time);
    for (const old of backups.slice(8)) {
      try { fs.unlinkSync(old.full); } catch {}
    }
    return backupPath;
  }

  function getWellbeing() {
    const row = db.prepare('SELECT * FROM wellbeing WHERE id=1').get();
    return {
      dailyMinutes: Number(row.daily_minutes),
      breakEveryMinutes: Number(row.break_every_minutes),
      windDownHour: Number(row.wind_down_hour),
      largeText: Boolean(row.large_text),
      reducedMotion: Boolean(row.reduced_motion),
    };
  }

  function saveWellbeing(value, allowLimits = false) {
    const next = normalizeWellbeing(value, getWellbeing(), allowLimits);
    db.prepare(`UPDATE wellbeing SET daily_minutes=?,break_every_minutes=?,wind_down_hour=?,large_text=?,reduced_motion=? WHERE id=1`)
      .run(next.dailyMinutes, next.breakEveryMinutes, next.windDownHour, next.largeText ? 1 : 0, next.reducedMotion ? 1 : 0);
    return next;
  }

  function listWorkspaceDocuments() {
    return db.prepare(`SELECT id,kind,title,prompt,content,created_at AS createdAt,updated_at AS updatedAt
                       FROM workspace_documents ORDER BY updated_at DESC LIMIT 50`).all();
  }

  function saveWorkspaceDocument(input) {
    const existing = input?.id
      ? db.prepare(`SELECT id,kind,title,prompt,content,created_at AS createdAt,updated_at AS updatedAt
                    FROM workspace_documents WHERE id=?`).get(String(input.id))
      : null;
    const document = normalizeWorkspaceDocument(input, existing);
    db.prepare(`INSERT INTO workspace_documents(id,kind,title,prompt,content,created_at,updated_at)
                VALUES(?,?,?,?,?,?,?)
                ON CONFLICT(id) DO UPDATE SET
                  kind=excluded.kind,title=excluded.title,prompt=excluded.prompt,content=excluded.content,updated_at=excluded.updated_at`)
      .run(document.id, document.kind, document.title, document.prompt, document.content, document.createdAt, document.updatedAt);
    return document;
  }

  function usageDayKey(date = new Date()) {
    return date.toISOString().slice(0, 10);
  }

  function addUsageSeconds(seconds) {
    const safeSeconds = Math.max(0, Math.min(300, Math.round(Number(seconds) || 0)));
    if (!safeSeconds) return;
    const day = usageDayKey();
    db.prepare(`INSERT INTO usage_daily(day,active_seconds) VALUES(?,?)
                ON CONFLICT(day) DO UPDATE SET active_seconds=active_seconds+excluded.active_seconds`).run(day, safeSeconds);
  }

  function getUsageStatus(now = new Date()) {
    const day = usageDayKey(now);
    const row = db.prepare('SELECT active_seconds FROM usage_daily WHERE day=?').get(day);
    const wellbeing = getWellbeing();
    const usedSeconds = Number(row?.active_seconds || 0);
    const dailyLimitSeconds = wellbeing.dailyMinutes * 60;
    const breakSeconds = wellbeing.breakEveryMinutes * 60;
    const localHour = now.getHours();
    return {
      day,
      usedMinutes: Math.floor(usedSeconds / 60),
      usedSeconds,
      dailyLimitMinutes: wellbeing.dailyMinutes,
      remainingMinutes: Math.max(0, Math.ceil((dailyLimitSeconds - usedSeconds) / 60)),
      dailyLimitReached: usedSeconds >= dailyLimitSeconds,
      breakDue: breakSeconds > 0 && usedSeconds > 0 && usedSeconds % breakSeconds < 60,
      windDownActive: localHour >= wellbeing.windDownHour,
    };
  }

  async function migrateLegacyFiles() {
    const legacyWellbeing = path.join(userDataDir, 'wellbeing.json');
    try {
      const raw = JSON.parse(await fsp.readFile(legacyWellbeing, 'utf8'));
      saveWellbeing(raw, true);
      await fsp.rename(legacyWellbeing, legacyWellbeing + '.migrated').catch(() => {});
    } catch {}

    const legacyDir = path.join(userDataDir, 'workspace-documents');
    try {
      const entries = await fsp.readdir(legacyDir, { withFileTypes: true });
      for (const entry of entries) {
        if (!entry.isFile() || !entry.name.endsWith('.project.json')) continue;
        try {
          const raw = JSON.parse(await fsp.readFile(path.join(legacyDir, entry.name), 'utf8'));
          saveWorkspaceDocument(raw);
        } catch {}
      }
      await fsp.rename(legacyDir, legacyDir + '.migrated').catch(() => {});
    } catch {}
  }

  function close() {
    try { createBackup('shutdown'); } catch {}
    db.close();
  }

  return {
    databasePath,
    migrateLegacyFiles,
    createBackup,
    getWellbeing,
    saveWellbeing,
    listWorkspaceDocuments,
    saveWorkspaceDocument,
    addUsageSeconds,
    getUsageStatus,
    close,
  };
}

module.exports = { openKidOSDataStore, DEFAULT_WELLBEING };
