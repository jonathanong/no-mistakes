import { greet } from "../lib/greeting.mts";

export function handleRequest(name: string) {
  return greet(name);
}
