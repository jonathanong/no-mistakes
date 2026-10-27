export async function shapes(locator) {
  await locator.isEnabled().catch(() => {
    return false;
  });
  await locator.isChecked().catch(async () => true);
  await locator.isVisible().catch(function () {
    return true;
  });
  await locator.isHidden()?.catch(() => false);
  await locator["isVisible"]().catch(() => false);
}

export async function fromTry(locator) {
  try {
    if (locator) {
      return await locator.isVisible();
    }
    return await locator.isEnabled();
  } catch (error) {
    console.error(error);
    return false;
  }
}
