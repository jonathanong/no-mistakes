declare function read(...args: any[]): Promise<any>;
const MAX_PAGES = 3;
export async function bounded() {
  for (let page = 0; page < MAX_PAGES; page++) await read(page);
}
export async function control(count: number) {
  for (let page = 0; page < count; page++) await read(page);
}
export async function literalBound() {
  for (let page = 0; page < 4; page++) await read(page);
}
export async function controlReset() {
  for (let page = 0; page < MAX_PAGES; page++) { await read(page); page = 0; }
}
export async function controlShadow(MAX_PAGES: number) {
  for (let page = 0; page < MAX_PAGES; page++) await read(page);
}
export async function controlUnsafe() {
  for (let page = 1e100; page < 1e101; page++) await read(page);
}
export async function nested(ids: string[]) {
  for (let page = 0; page < MAX_PAGES; page++) {
    for (const id of ids) await read(id);
  }
}
const START = 1;
const STEP = 2;
export async function finiteCounters() {
  for (let page = START; page < MAX_PAGES; page++) await read(page);
  for (let page = 0; page <= MAX_PAGES; page += STEP) await read(page);
  for (let page = MAX_PAGES; page > 0; page--) await read(page);
  for (let page = MAX_PAGES; page >= START; page -= STEP) await read(page);
  for (let page = 2; page >= -1; page--) await read(page);
}
export async function controlDirections(step: number) {
  for (let page = 0; page < MAX_PAGES; page--) await read(page);
  for (let page = MAX_PAGES; page > 0; page++) await read(page);
  for (let page = 0; page < MAX_PAGES; page += 0) await read(page);
  for (let page = 0; page < MAX_PAGES; page += step) await read(page);
}
export async function controlSelf() {
  const page = 2;
  for (let page = 0; page <= page; page++) await read(page);
}
