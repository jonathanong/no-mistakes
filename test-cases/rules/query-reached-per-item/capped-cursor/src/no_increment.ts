import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; attempt < maximum;) { const chunk = await read(token); token = chunk.next; }
}
