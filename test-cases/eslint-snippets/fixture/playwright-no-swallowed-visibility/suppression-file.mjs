// no-mistakes-disable-file playwright-no-swallowed-visibility
export async function missing(locator) {
  return await locator.isVisible().catch(() => false);
}
