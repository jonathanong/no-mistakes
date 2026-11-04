import {read, handle} from "./sink";
export async function run(maximum: number, ready: boolean) {
  let state = {after: undefined}; for (let attempt = 0; attempt < maximum; attempt++) { const chunk = await read(state.after); state.after = chunk.next; }
}
