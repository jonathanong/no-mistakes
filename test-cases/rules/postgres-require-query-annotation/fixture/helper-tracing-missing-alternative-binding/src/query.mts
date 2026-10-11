import { write } from "@app/db";

export function annotatedStaysClean() {
  write("/* posts/list */ SELECT 1");
}

export function missingAnnotation() {
  write("SELECT id FROM posts"); // finding:unannotated
}

export function divergentAlternativeBinding(flag: boolean) {
  // onlyLeft exists on one side of the alternative. Join must not panic or
  // keep that arm's annotation as proof for the later read.
  const chosen = flag ? (onlyLeft = "/* divergent */ SELECT 1") : "SELECT 2";
  write(onlyLeft); // unanalyzable:missing-alternative-binding
  return chosen;
}

export function divergentIfElse(flag: boolean) {
  // A block binding that exists on only one if/else side is not a proven annotation.
  let query = "SELECT 1";
  if (flag) {
    query = "/* if branch */ SELECT 1";
  } else {
    const onlyElse = "/* else branch */ SELECT 2";
    query = onlyElse;
  }
  write(query); // Both branches assign complete annotated SQL.
}
