import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read({from: true ? 0 : token}); token = chunk.next; }
}
