declare function read(...args: any[]): Promise<any>;
export async function negativeCounter() {
  for (let page = NEGATIVE_START; page < 0; page++) await read(page);
}
const NEGATIVE_START = -2;
