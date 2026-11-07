declare function read(...args: any[]): Promise<any>;
const MAX = 3;
export async function controlCounterPattern(seed: number[], count: number) {
  for (let [page] = seed; page < count; page++) await read(page);
}
export async function controlCounterNoInit(count: number) {
  for (let page: number; page < count; page++) await read(page);
}
export async function controlCounterComparison(count: number) {
  for (let page = 0; page !== count; page++) await read(page);
}
declare function advance(): void;
export async function controlCounterUpdate() {
  for (let page = 0; page < MAX; advance()) await read(page);
}
export async function controlCounterBodyUpdate() {
  for (let page = 0; page < MAX; page++) { await read(page); page--; }
}
export async function objectUpdateCounter(holder: { count: number }) {
  for (let page = 0; page < MAX; page++) { holder.count++; await read(page); }
}
