export async function consume(callback: () => Promise<void>) {
  outer: for (;;) { break outer; }
  await callback();
}
