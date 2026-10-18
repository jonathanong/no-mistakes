import { read } from "@example/db";
import { unionOf } from "./union.mts";

export async function literalArray() {
  return read(unionOf(["relation_a", "relation_b"]));
}

export async function defaultIsEmpty() {
  return read(unionOf());
}

// The loop variable is a parameter the walk cannot unroll over.
export async function unknownArray(tables: string[]) {
  return read(unionOf(tables));
}

// Seventeen elements is past the cap of sixteen.
export async function longArray() {
  return read(
    unionOf([
      "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8",
      "r9", "r10", "r11", "r12", "r13", "r14", "r15", "r16",
    ]),
  );
}
