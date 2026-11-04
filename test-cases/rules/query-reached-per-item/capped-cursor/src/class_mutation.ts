import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    class Reset { static run() { token = 0; } } Reset.run(); const chunk = await read(token); token = chunk.next;
  }
}
