import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { spawnSync } from 'node:child_process';

const args = process.argv.slice(2);
function take(flag) {
  const i = args.indexOf(flag);
  if (i < 0 || i + 1 >= args.length) throw new Error(`Missing ${flag}`);
  const value = args[i + 1];
  args.splice(i, 2);
  return value;
}

const count = Number.parseInt(take('--count'), 10);
const artifact = take('--artifact');
if (!Number.isInteger(count) || count < 1 || count > 100) throw new Error('--count must be 1..100');
if (args.length === 0) throw new Error('A command is required');
const [command, ...commandArgs] = args;

let completed = 0;
let failedIteration = null;
let exitCode = 0;
for (let iteration = 1; iteration <= count; iteration += 1) {
  const result = spawnSync(command, commandArgs, { stdio: 'inherit', shell: false });
  completed = iteration;
  const status = result.status ?? 1;
  if (status !== 0) {
    failedIteration = iteration;
    exitCode = status;
    break;
  }
}

const output = {
  passed: failedIteration === null,
  count,
  completed,
  failed_iteration: failedIteration,
  command: [command, ...commandArgs],
  exit_code: exitCode,
};
mkdirSync(dirname(artifact), { recursive: true });
writeFileSync(artifact, `${JSON.stringify(output, null, 2)}\n`, 'utf8');
process.exit(exitCode);
