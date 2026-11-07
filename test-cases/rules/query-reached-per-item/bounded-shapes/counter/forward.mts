declare function read(...args: any[]): Promise<any>;
export async function forwardCounter() {
  for (let page = 0; page < LATER_MAX; page++) await read(page);
}
const LATER_MAX = 3;
