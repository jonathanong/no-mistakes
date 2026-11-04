import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(globalContinuation); globalContinuation = chunk.next; }
}
