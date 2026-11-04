import {read, handle} from "./sink";
import {pageRead} from "./page-read";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; attempt < 5; attempt++) { const chunk = await read(token); token = chunk.next; }
}
