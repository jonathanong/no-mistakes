// Four retained argument slots and three lexical values; repeated writes hit one slot.
import { query } from "@bench/db";
declare const choose: boolean;
const lexical00 = 0;
const lexical01 = 1;
const lexical02 = 2;

function payload(
  flag: boolean,
  target: string,
  untouched0: string,
  untouched1: string,
  untouched2: string
) {
  const reader = () => arguments[1];
  flag
    ? ((arguments[1] = "intermediate left"), (arguments[1] = "SELECT 1 /* left */"))
    : ((arguments[1] = "intermediate right"), (arguments[1] = "SELECT 2 /* right */"));
  return reader;
}

const reader = payload(choose, "initial", "value-0", "value-1", "value-2");
export const selected = query(reader());
