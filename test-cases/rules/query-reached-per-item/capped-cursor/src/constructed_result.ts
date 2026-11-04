import {read} from "./sink";
declare class Mutator { constructor(value: unknown); }
export async function run(maximum: number) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); new Mutator(chunk); token = chunk.next; }
}
