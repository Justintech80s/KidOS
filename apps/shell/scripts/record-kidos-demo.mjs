import { chromium } from 'playwright';
import { mkdir, rm, rename } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const root = resolve(new URL('../../..', import.meta.url).pathname);
const outputDir = resolve(root, 'artifacts/demo-video');
const webmPath = resolve(outputDir, 'KidOS-Demo.webm');
const mp4Path = resolve(outputDir, 'KidOS-Demo.mp4');

await rm(outputDir, { recursive: true, force: true });
await mkdir(outputDir, { recursive: true });

const server = spawn(
  process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm',
  ['--filter', '@kidos/shell', 'dev', '--host', '127.0.0.1'],
  { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] },
);

let ready = false;
const waitForReady = new Promise((resolveReady, rejectReady) => {
  const timeout = setTimeout(() => rejectReady(new Error('KidOS dev server did not start in time')), 30000);
  const onData = (chunk) => {
    const text = chunk.toString();
    process.stdout.write(text);
    if (text.includes('127.0.0.1:') || text.includes('ready in')) {
      clearTimeout(timeout);
      ready = true;
      resolveReady();
    }
  };
  server.stdout.on('data', onData);
  server.stderr.on('data', (chunk) => process.stderr.write(chunk));
  server.on('exit', (code) => {
    if (!ready) rejectReady(new Error(`KidOS dev server exited early with code ${code}`));
  });
});

await waitForReady;

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  recordVideo: { dir: outputDir, size: { width: 1440, height: 900 } },
});

const page = await context.newPage();
await page.goto('http://127.0.0.1:1420/?demo=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(24500);

const video = page.video();
await context.close();
const recordedPath = await video.path();
await browser.close();

if (recordedPath !== webmPath) {
  await rename(recordedPath, webmPath);
}

server.kill('SIGTERM');
await new Promise((resolveServer) => {
  if (server.exitCode !== null || server.signalCode !== null) {
    resolveServer();
    return;
  }
  const timer = setTimeout(() => resolveServer(), 3000);
  server.once('exit', () => {
    clearTimeout(timer);
    resolveServer();
  });
});

const ffmpeg = spawn(
  'ffmpeg',
  ['-y', '-i', webmPath, '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', mp4Path],
  { cwd: root, stdio: 'inherit' },
);

await new Promise((resolveFfmpeg, rejectFfmpeg) => {
  ffmpeg.on('exit', (code) => {
    if (code === 0) resolveFfmpeg();
    else rejectFfmpeg(new Error(`ffmpeg exited with code ${code}`));
  });
});

console.log(`KidOS demo video created: ${mp4Path}`);
process.exit(0);
