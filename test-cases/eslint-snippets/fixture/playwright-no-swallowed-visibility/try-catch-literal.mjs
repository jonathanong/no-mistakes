export async function missing(locator) {
  try {
    return await locator.isVisible();
  } catch {
    return false;
  }
}

export async function present(locator) {
  try {
    return await locator.isChecked();
  } catch (error) {
    return true;
  }
}
