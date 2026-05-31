import { parseHashDate } from "./hash#utils.mts";

export function testHashConsumer() {
  return parseHashDate("2026-05-31");
}

