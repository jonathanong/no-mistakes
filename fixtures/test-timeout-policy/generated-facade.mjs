import { mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Exercise the generated public module through an actual worker import. */
export async function withGeneratedFacade(source, inspect) {
  const directory = await mkdtemp(join(tmpdir(), 'parent-node-facade-'));
  try {
    await symlink(fileURLToPath(new URL('../../node_modules', import.meta.url)), join(directory, 'node_modules'));
    const module = join(directory, 'facade.mjs');
    await writeFile(module, source);
    await inspect(await import(module));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}
