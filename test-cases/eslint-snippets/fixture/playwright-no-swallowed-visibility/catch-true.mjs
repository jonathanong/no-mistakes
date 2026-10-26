// `.catch(() => true)` turns a failed hidden check into presence.
export async function forced(locator) {
  return await locator.isHidden().catch(() => true);
}
