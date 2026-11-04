import {read, handle} from "./sink";
import {pageRead} from "./page-read";
export async function run(maximum: number, items: unknown[]) {
  for (const item of items) { let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); token = chunk.next; } }
}
