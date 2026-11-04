import {read} from "./sink";
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    var maximum = Infinity;
    const chunk = await read({from: token});
    token = chunk.next;
  }
}
