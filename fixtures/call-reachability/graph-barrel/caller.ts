import { createProgram as aliased } from "./barrel";
import { api, client, value } from "./source";

export function run() {
  aliased();
  client.create();
  api.run();
  return value;
}
