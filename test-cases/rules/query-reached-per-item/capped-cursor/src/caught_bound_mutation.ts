import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    try { maximum = Infinity; } catch {} const chunk = await read(token); token = chunk.next;
  }
}
