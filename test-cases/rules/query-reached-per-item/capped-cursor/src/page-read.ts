import {read} from "./sink";
export async function pageRead(token: unknown, items: unknown[]) {
  for (const item of items) await read(item);
  return read(token);
}
