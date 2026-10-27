export async function absent(locator) {
  await locator.isVisible().catch(() => !1);
  await locator.isHidden().catch(() => !0);
}

export async function fromTry(locator) {
  try {
    return await locator.isEnabled();
  } catch {
    return !1;
  }
}
