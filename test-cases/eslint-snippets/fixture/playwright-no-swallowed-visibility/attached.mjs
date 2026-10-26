// isAttached is outside the default method list.
export async function attached(locator) {
  return await locator.isAttached().catch(() => false);
}
