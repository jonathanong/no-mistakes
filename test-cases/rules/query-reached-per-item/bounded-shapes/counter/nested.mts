declare function read(...args: any[]): Promise<any>;
const MAX = 3;
export async function nestedQueries(ids: string[]) {
  for (let page = 0; page < MAX; page++) {
    await read(page);
    for (const id of ids) await read(id);
  }
}
export async function taggedCounter() {
  for (let page = 0; page < MAX; page++) await read`SELECT ${page}`;
}
export async function controlEval() {
  for (let page = 0; page < MAX; page++) {
    eval('page = 0');
    await read(page);
  }
}
export async function controlTaggedReset() {
  for (let page = 0; page < MAX; page++) await read`SELECT ${page = 0}`;
}
export async function controlLoopWrite(offsets: number[]) {
  for (let page = 0; page < MAX; page++) {
    await read(page);
    for (page of offsets) await read(page);
  }
}
