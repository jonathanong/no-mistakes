import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  for (let outer = 0; outer < maximum; outer++) { let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); token = chunk.next; } }
}
