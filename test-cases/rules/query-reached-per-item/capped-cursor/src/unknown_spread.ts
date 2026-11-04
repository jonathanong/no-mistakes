import {read} from "./sink";
export async function run(maximum: number, options: object) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read({from: token, ...options}); token = chunk.next; }
}
