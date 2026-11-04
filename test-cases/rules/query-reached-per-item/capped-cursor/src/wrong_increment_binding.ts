import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  let token; let other = 0; for (let attempt = 0; attempt < maximum; other++) { const chunk = await read(token); token = chunk.next; }
}
