import { used, helper } from "./util";

const args: [string, { x: number }] = ["a", { x: 2 }];

export function run() {
  used("hi", { x: 1 });
  // Spread call — exercises has_spread / arg_count without per-arg shapes.
  used(...args);
  helper();
}

// Top-level call site — its `caller` is null (no enclosing function).
used("top", { x: 0 });
