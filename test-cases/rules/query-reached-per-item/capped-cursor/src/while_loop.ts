import {read, handle} from "./sink";
import {pageRead} from "./page-read";
export async function run(maximum: number, items: unknown[]) {
  let token; while (maximum) { const chunk = await read(token); token = chunk.next; }
}
