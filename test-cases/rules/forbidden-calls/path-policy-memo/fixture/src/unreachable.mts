// Matching calls outside the selected roots must never become findings.
export function unreachable() {
  setTimeout(() => {}, 7);
  setTimeout(() => {}, 8);
}
