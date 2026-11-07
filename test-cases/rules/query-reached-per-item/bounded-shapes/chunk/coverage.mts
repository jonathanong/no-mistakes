declare function read(...args: any[]): Promise<any>;
export async function literalChunk(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) await read(ids.slice(i, i + 2));
}
export async function controlChunkUnary(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) await read(ids.slice(i, i + -(-2)));
}
export async function controlChunkMissingEnd(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) await read(ids.slice(i));
}
export async function controlChunkEndIndex(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) await read(ids.slice(i, i));
}
export async function controlChunkComputed(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) await read(ids['slice'](i, i + 2));
}
export async function controlChunkLengthComparison(ids: string[]) {
  for (let i = 0; i < ids.size; i += 2) await read(ids.slice(i, i + 2));
}
export async function controlChunkReceiver(ids: { values: string[] }) {
  for (let i = 0; i < ids.values.length; i += 2) await read(ids.values.slice(i, i + 2));
}
export async function controlChunkUpdateTarget(ids: string[], state: { i: number }) {
  for (let i = 0; i < ids.length; state.i += 2) await read(ids.slice(i, i + 2));
}
export async function controlChunkNestedCallback(ids: string[]) {
  for (let i = 0; i < ids.length; i += 2) ids.map(id => read(ids.slice(i, i + 2)));
}
export async function controlChunkArrowStep(ids: string[]) {
  const STEP = () => 2;
  for (let i = 0; i < ids.length; i += STEP()) await read(ids.slice(i, i + 2));
}
