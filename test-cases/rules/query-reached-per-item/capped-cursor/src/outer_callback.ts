import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  await Promise.all(items.map(async item => { let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); token = chunk.next; } }));
}
