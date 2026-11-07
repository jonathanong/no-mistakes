declare function read(...args: any[]): Promise<any>;
const CHUNK = 100;
export async function bounded(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) await read(ids.slice(i, i + CHUNK));
}
export async function control(ids: string[]) {
  for (let i = 0; i < ids.length; i++) await read(ids[i]);
}
export async function sqlBatch(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) await read('SELECT * FROM items WHERE id = ANY($1)', [ids.slice(i, i + CHUNK)]);
}
export async function controlSparse(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) await read(ids[i]);
}
export async function controlDynamic(ids: string[], size: number) {
  for (let i = 0; i < ids.length; i += size) await read(ids.slice(i, i + size));
}
export async function controlMismatch(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) await read(ids.slice(i, i + 1));
}
export async function controlMixed(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) {
    await read(ids.slice(i, i + CHUNK));
    await read(ids[i]);
  }
}
export async function controlOuter(groups: string[][]) {
  for (const ids of groups) {
    for (let i = 0; i < ids.length; i += CHUNK) await read(ids.slice(i, i + CHUNK));
  }
}
export async function controlChunkShadow(ids: string[]) {
  for (let i = 0; i < ids.length; i += CHUNK) {
    const CHUNK = 1;
    await read(ids.slice(i, i + CHUNK));
  }
}
