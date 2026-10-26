export async function mixed(locator) {
  // no-mistakes-disable-next-line playwright-no-swallowed-visibility
  await locator.isVisible().catch(() => false);
  await locator.isHidden().catch(() => true); // no-mistakes-disable-line playwright-no-swallowed-visibility
  // no-mistakes-disable-next-line no-argv-main-guard
  await locator.isChecked().catch(() => false);
  await locator.isEnabled().catch(() => false);
}
