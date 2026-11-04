import {read} from "./sink";
export async function run(maximum: number) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read({from: maximum ? token : 0}); token = chunk.next; }
}
