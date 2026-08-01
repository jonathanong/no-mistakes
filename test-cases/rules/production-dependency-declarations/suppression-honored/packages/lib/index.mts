import { doThing } from "@acme/tool"; // no-mistakes-disable-line production-dependency-declarations

export function helper() {
  return doThing();
}
