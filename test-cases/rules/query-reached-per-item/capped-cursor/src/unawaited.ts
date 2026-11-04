import {read, handle} from "./sink";
import {pageRead} from "./page-read";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = read(token); token = chunk.next; }
}
