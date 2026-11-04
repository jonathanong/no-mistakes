import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  let token; for (var attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); token = chunk.next; }
}
