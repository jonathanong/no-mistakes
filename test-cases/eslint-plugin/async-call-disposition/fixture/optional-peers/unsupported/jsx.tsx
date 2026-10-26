export async function consume(callback: () => Promise<void>) {
  const element = <Component />;
  await callback();
}
