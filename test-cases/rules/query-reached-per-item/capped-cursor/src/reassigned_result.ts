import {read} from "./sink";
export async function run(maximum: number, items: unknown[]) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { let chunk = await read(token); chunk = await read(0); token = chunk.next; }
}
