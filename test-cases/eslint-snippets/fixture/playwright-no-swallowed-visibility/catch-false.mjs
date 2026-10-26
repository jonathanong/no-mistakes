// `.catch(() => false)` turns a failed visibility check into absence.
export async function missing(locator) {
  return await locator.isVisible().catch(() => false);
}
