export function outer() {
  async function nested() {
    await import("./uncalled.mts");
  }

  return nested;
}
