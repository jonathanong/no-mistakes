import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const alias = token; const chunk = await read(alias); token = chunk.next; }
}
