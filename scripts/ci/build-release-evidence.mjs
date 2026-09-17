import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

const args = process.argv.slice(2);
function take(flag) {
  const i = args.indexOf(flag);
  if (i < 0 || i + 1 >= args.length) throw new Error(`Missing ${flag}`);
  return args[i + 1];
}
const sha = take('--sha');
const checksPath = take('--checks');
const outputPath = take('--output');
const payload = JSON.parse(readFileSync(checksPath, 'utf8'));
const checks = payload.check_runs ?? [];

for (const check of checks) {
  if (check.head_sha && check.head_sha !== sha) {
    throw new Error(`Check ${check.name} belongs to ${check.head_sha}, not ${sha}`);
  }
}

function statusFor(pattern) {
  const matching = checks.filter((check) => pattern.test(check.name));
  if (matching.length === 0) return 'not_run';
  if (matching.some((check) => check.status !== 'completed')) return 'pending';
  if (matching.some((check) => check.conclusion !== 'success' && check.conclusion !== 'skipped')) return 'failed';
  return matching.some((check) => check.conclusion === 'success') ? 'passed' : 'not_run';
}

const manifest = {
  commit_sha: sha,
  ref: process.env.GITHUB_REF ?? '',
  generated_at: new Date().toISOString(),
  codeql: statusFor(/CodeQL/i),
  dependency_security: statusFor(/dependency-security|Dependency Security/i),
  unit_tests: statusFor(/windows-release|verify-shell|Run JavaScript tests/i),
  guardian_contract: statusFor(/windows-release|guardian/i),
  installer_upgrade: statusFor(/installer-upgrade|Installer Upgrade/i),
  fault_injection: statusFor(/fault-injection/i),
  flake_check: statusFor(/critical-contract-repeat|Flake Check/i),
  post_merge_smoke: statusFor(/exact-main-sha-smoke|Post-Merge Smoke/i),
  hyperv_visual_validation: statusFor(/visual-guest-validation|Interactive Windows Session/i),
};

const cloudRequired = [
  manifest.codeql,
  manifest.dependency_security,
  manifest.unit_tests,
  manifest.guardian_contract,
  manifest.installer_upgrade,
  manifest.fault_injection,
  manifest.flake_check,
  manifest.post_merge_smoke,
];
manifest.cloud_ci_ready = cloudRequired.every((status) => status === 'passed');
manifest.release_ready = manifest.cloud_ci_ready && manifest.hyperv_visual_validation === 'passed';

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
