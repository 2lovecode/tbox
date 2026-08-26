// Generate a SHA256SUMS file for every regular file under a directory.
// Used by the Release workflow to publish checksums next to the installer
// bundles produced by `tauri-action`.
//
// Usage: node scripts/checksums.mjs <source-dir> <output-file>
//
// The output file is written OUTSIDE the source directory (or at least after
// the walk) so it never lists itself.

import { createReadStream, promises as fs } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';

const [src, out] = process.argv.slice(2);
if (!src || !out) {
  console.error('usage: node scripts/checksums.mjs <source-dir> <output-file>');
  process.exit(1);
}

async function* walk(dir) {
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walk(p);
    } else if (entry.isFile()) {
      yield p;
    }
  }
}

function sha256(file) {
  return new Promise((resolve, reject) => {
    const hash = createHash('sha256');
    createReadStream(file)
      .on('error', reject)
      .on('data', (chunk) => hash.update(chunk))
      .on('end', () => resolve(hash.digest('hex')));
  });
}

const files = [];
for await (const file of walk(src)) files.push(file);
files.sort();

const lines = [];
for (const file of files) {
  const rel = path.relative(src, file).split(path.sep).join('/');
  lines.push(`${await sha256(file)}  ${rel}`);
}

await fs.writeFile(out, `${lines.join('\n')}${lines.length ? '\n' : ''}`);
console.log(`wrote ${out} (${lines.length} files)`);
