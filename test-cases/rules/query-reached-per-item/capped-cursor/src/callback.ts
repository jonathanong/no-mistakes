import {read, handle} from "./sink";
import {pageRead} from "./page-read";
export async function run(maximum: number, items: unknown[]) {
  let token; await Promise.all(items.map(async item => { const chunk = await read(token); token = chunk.next; }));
}
