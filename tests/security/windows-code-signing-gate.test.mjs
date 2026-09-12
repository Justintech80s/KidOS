import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const workflow = readFileSync(
  new URL('../../.github/workflows/ci.yml', import.meta.url),
  'utf8',
);

assert.match(
  workflow,
  /production-signing-gate:/,
  'Windows release CI must define an isolated production-signing gate.',
);
assert.match(
  workflow,
  /refs\/tags\/v/,
  'Production signing must be isolated from the experimental main-branch release channel.',
);
assert.match(
  workflow,
  /KIDOS_WINDOWS_CERTIFICATE_BASE64/,
  'Production signing must require the base64-encoded Authenticode certificate secret.',
);
assert.match(
  workflow,
  /KIDOS_WINDOWS_CERTIFICATE_PASSWORD/,
  'Production signing must require the certificate password secret.',
);
assert.match(
  workflow,
  /signtool(?:\.exe)?[^\r\n]*sign[^\r\n]*\/fd SHA256/i,
  'Production signing must use SHA-256 file digests.',
);
assert.match(
  workflow,
  /\/tr https:\/\/timestamp\.digicert\.com[^\r\n]*\/td SHA256/i,
  'Production signing must use an RFC3161 timestamp with SHA-256.',
);
assert.match(
  workflow,
  /signtool(?:\.exe)?[^\r\n]*verify[^\r\n]*\/pa[^\r\n]*\/v/i,
  'CI must verify the signed installer with Windows Authenticode policy before promotion.',
);
assert.match(
  workflow,
  /Get-FileHash[^\r\n]*SHA256/i,
  'Production release must generate a SHA-256 integrity manifest after signing.',
);
assert.match(
  workflow,
  /KidOS-SHA256SUMS\.txt/,
  'Production release must publish the SHA-256 integrity manifest.',
);
assert.match(
  workflow,
  /KidOS-Windows-10-11-x64-Signed/,
  'Production release must emit a distinct signed installer artifact.',
);
assert.match(
  workflow,
  /gh release (?:create|upload)/,
  'Production promotion must publish only after the signing and verification steps succeed.',
);

console.log('KidOS Windows production signing gate regression checks passed.');
