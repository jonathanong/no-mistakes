import {read} from "./sink";
export async function run(maximum: number) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read({...(maximum ? {from: token} : {})}); token = chunk.next; }
}
