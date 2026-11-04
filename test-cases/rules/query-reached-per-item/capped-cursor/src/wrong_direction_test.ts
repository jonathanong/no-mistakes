import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  let token; for (let attempt = 0; attempt >= 0; attempt++) { const chunk = await read(token); token = chunk.next; }
}
