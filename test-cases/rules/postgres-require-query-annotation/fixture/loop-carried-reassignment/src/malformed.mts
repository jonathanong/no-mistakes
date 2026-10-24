import { read } from "@example/db";
export function broken(xs) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) { read(q); q = "SELECT 2"; }
}
const broken = ;
