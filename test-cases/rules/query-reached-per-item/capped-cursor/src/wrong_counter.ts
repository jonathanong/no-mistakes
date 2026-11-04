import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; maximum < 5; attempt++) { const chunk = await read(token); token = chunk.next; }
}
