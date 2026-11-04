import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    do { token = 0; } while (false); const chunk = await read(token); token = chunk.next;
  }
}
