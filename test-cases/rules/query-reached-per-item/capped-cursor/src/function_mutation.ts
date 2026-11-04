import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    function reset() { token = 0; } reset(); const chunk = await read(token); token = chunk.next;
  }
}
