export async function consume(callback: () => Promise<void>) {
  using resource = acquire();
  await callback();
}
