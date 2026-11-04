import {read} from "./sink";
export async function run(maximum: number, other: unknown) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); (maximum ? chunk : other).reset(); token = chunk.next; }
}
