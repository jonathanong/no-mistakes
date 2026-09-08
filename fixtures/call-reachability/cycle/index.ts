export function start() {
  first();
}

function first() {
  second();
}

function second() {
  // This back edge intentionally protects cycle termination and shortest traces.
  first();
}
