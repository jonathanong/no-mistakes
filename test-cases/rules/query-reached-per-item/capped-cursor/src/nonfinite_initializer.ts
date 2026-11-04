import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 1e999; attempt < maximum; attempt++) {
    const chunk = await read(token);
    token = chunk.next;
  }
}
