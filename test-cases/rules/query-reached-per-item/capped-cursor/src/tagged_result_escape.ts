import {read} from "./sink";
declare function mutate(strings: TemplateStringsArray, value: unknown): void;
export async function run(maximum: number) {
  let token;
  for (let attempt = 0; attempt < maximum; attempt++) {
    const chunk = await read(token); mutate`${chunk}`; token = chunk.next;
  }
}
