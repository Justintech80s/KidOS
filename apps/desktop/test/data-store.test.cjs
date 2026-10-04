const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { openKidOSDataStore } = require('../electron/data-store.cjs');

function withStore(fn) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'kidos-store-'));
  const store = openKidOSDataStore(dir);
  return Promise.resolve(fn(store, dir)).finally(() => {
    try { store.close(); } catch {}
    fs.rmSync(dir, { recursive: true, force: true });
  });
}

test('KidOS SQLite store persists projects and rolling backups', async () => withStore((store, dir) => {
  const saved = store.saveWorkspaceDocument({
    kind: 'story',
    title: 'Moon Story',
    prompt: 'Write a moon story',
    content: 'Once upon a Moon.',
  });
  assert.ok(saved.id);
  assert.equal(store.listWorkspaceDocuments()[0].content, 'Once upon a Moon.');

  const backup = store.createBackup('test');
  assert.ok(backup);
  assert.equal(fs.existsSync(backup), true);
  assert.equal(path.dirname(backup), path.join(dir, 'backups'));
}));

test('child comfort saves cannot raise parent-controlled limits', async () => withStore((store) => {
  const parent = store.saveWellbeing({
    dailyMinutes: 45,
    breakEveryMinutes: 15,
    windDownHour: 19,
    largeText: false,
    reducedMotion: false,
  }, true);
  assert.equal(parent.dailyMinutes, 45);

  const child = store.saveWellbeing({
    dailyMinutes: 600,
    breakEveryMinutes: 120,
    windDownHour: 23,
    largeText: true,
    reducedMotion: true,
  }, false);

  assert.equal(child.dailyMinutes, 45);
  assert.equal(child.breakEveryMinutes, 15);
  assert.equal(child.windDownHour, 19);
  assert.equal(child.largeText, true);
  assert.equal(child.reducedMotion, true);
}));

test('usage accounting drives remaining time and daily-limit enforcement state', async () => withStore((store) => {
  store.saveWellbeing({
    dailyMinutes: 15,
    breakEveryMinutes: 10,
    windDownHour: 23,
    largeText: false,
    reducedMotion: false,
  }, true);

  for (let minute = 0; minute < 14; minute += 1) {
    store.addUsageSeconds(60);
  }
  let status = store.getUsageStatus(new Date('2026-10-04T12:00:00'));
  assert.equal(status.usedMinutes, 14);
  assert.equal(status.dailyLimitReached, false);

  store.addUsageSeconds(60);
  status = store.getUsageStatus(new Date('2026-10-04T12:01:00'));
  assert.equal(status.usedMinutes, 15);
  assert.equal(status.dailyLimitReached, true);
  assert.equal(status.remainingMinutes, 0);
}));
