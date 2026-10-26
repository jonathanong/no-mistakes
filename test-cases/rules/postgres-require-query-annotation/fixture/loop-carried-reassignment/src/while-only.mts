import { read } from "@example/db";

export function repeat(again: boolean) {
  let q = "/* named */ SELECT 1";
  while (again) {
    read(q); // missing: the while keyword alone must enable the syntax witness.
    q = "SELECT 2";
    console.log(again);
  }
}
export function repeatDo(again: boolean) {
  let q = "/* named */ SELECT 1";
  do {
    read(q); // missing: do-while uses the same cheap keyword gate.
    q = "SELECT 2";
  } while (again);
}
