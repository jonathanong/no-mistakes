import {read} from "./sink";
declare function mutate(value: unknown): void;
let alias;
export async function run(maximum: number) {
  let token; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(token); alias = chunk; token = chunk.next; }
}
