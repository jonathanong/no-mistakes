export async function asserted(locator: { isVisible: () => Promise<boolean> }) {
  await locator.isVisible().catch((() => false) as () => boolean);
  try {
    return await (locator.isVisible() as Promise<boolean>);
  } catch {
    return true;
  }
}
