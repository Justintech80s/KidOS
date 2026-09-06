import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

function read(path) {
  return readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');
}

const tauri = JSON.parse(read('apps/shell/src-tauri/tauri.conf.json'));
const hooks = read('apps/shell/src-tauri/windows/hooks.nsh');
const browserHost = read('apps/shell/src-tauri/src/lib.rs');
const lockdownConfig = read('crates/guardian-service/src/windows_lockdown/config.rs');
const guardianIpc = read('crates/guardian-host/src/ipc_server.rs');
const updater = read('crates/guardian-host/src/updater.rs');
const recovery = read('scripts/windows/kidos-recovery.ps1');

assert.equal(tauri.bundle?.windows?.nsis?.installMode, 'perMachine', 'KidOS must install per-machine with administrator approval.');
assert.match(tauri.app?.security?.csp ?? '', /default-src 'self'/, 'KidOS must have an explicit CSP.');
assert.doesNotMatch(tauri.app?.security?.csp ?? '', /default-src \*/, 'KidOS CSP must not allow every source.');

assert.match(hooks, /obj= LocalSystem/, 'Privileged KidOS services must run as LocalSystem.');
assert.match(hooks, /icacls\.exe/, 'Installer must harden KidOS service directories.');
assert.match(hooks, /KidOS Guardian Recovery/, 'Installer must register the recovery task.');
assert.match(hooks, /\/RU SYSTEM/, 'Recovery task must run under SYSTEM.');
assert.match(hooks, /publisher-thumbprint\.txt/, 'Signed production installs must pin their Windows publisher certificate.');
assert.doesNotMatch(hooks, /AutoAdminLogon|DefaultPassword/, 'Production installer must never enable Windows automatic logon.');

assert.match(browserHost, /matches!\(url\.scheme\(\), "https" \| "http"\)/, 'Safe Browser must restrict navigations to HTTP(S).');
assert.match(browserHost, /NewWindowResponse::Deny/, 'Safe Browser must deny uncontrolled popup windows.');
assert.match(browserHost, /guardian_ipc::evaluate_download/, 'Browser downloads must pass through Guardian policy.');

for (const tool of ['cmd.exe', 'powershell.exe', 'pwsh.exe', 'regedit.exe', 'wt.exe', 'wscript.exe', 'cscript.exe', 'mmc.exe']) {
  assert.match(lockdownConfig, new RegExp(tool.replace('.', '\\.')), `Assigned Access must reject ${tool}.`);
}
assert.match(guardianIpc, /prohibited_executable_name/, 'Guardian must independently validate approved desktop executables.');
assert.match(guardianIpc, /validate_standard_windows_account/, 'Guardian must validate the Windows child account.');

assert.match(updater, /https:\/\/github\.com\/Justintech80s\/KidOS\/releases\//, 'Updater must be pinned to the official KidOS GitHub release namespace.');
assert.match(updater, /Sha256/, 'Updater must hash downloaded installers with SHA-256.');
assert.match(updater, /Get-AuthenticodeSignature/, 'Updater must verify Windows Authenticode signatures.');
assert.match(updater, /pinned_publisher_thumbprint/, 'Updater must compare releases against the publisher certificate pinned at install time.');
assert.match(updater, /publisher-thumbprint\.txt/, 'Updater must load its publisher pin from protected Guardian storage.');
assert.match(updater, /candidate <= current/, 'Updater must reject rollback and same-version installs.');
assert.match(updater, /Guardian.*Updates|guardian_data_dir\(\).*Updates/s, 'Updater staging must remain under protected Guardian storage.');
assert.match(updater, /verify_staged_installer/, 'A staged update must be reverified before execution.');

const updaterCommands = [...updater.matchAll(/Command::new\(([^\n]+?)\)/g)].map((match) => match[1].trim());
assert.deepEqual(
  updaterCommands,
  ['"powershell.exe"', '&installer'],
  'The privileged updater may invoke only Windows signature verification and its already-verified staged installer.',
);
const signatureVerifier = updater.match(/fn verify_authenticode[\s\S]*?\n}\r?\n/)?.[0] ?? '';
assert.match(signatureVerifier, /Get-AuthenticodeSignature/, 'The updater signature-verification function must use Windows Authenticode.');
assert.match(signatureVerifier, /Command::new\("powershell\.exe"\)/, 'The only PowerShell updater command must be inside Authenticode verification.');
assert.match(updater, /verify_staged_installer\(&manifest, &installer\)\?[\s\S]*Command::new\(&installer\)[\s\S]*\.arg\("\/S"\)/, 'Installer execution must occur only after staged-file revalidation and only in silent install mode.');

assert.match(recovery, /Start-Service \$guardian/, 'Recovery must attempt to restore Guardian after service failure.');
assert.match(recovery, /classifier-service-failed/, 'Classifier failure must be tracked without silently weakening media safety.');

console.log('KidOS production hardening regression checks passed.');
