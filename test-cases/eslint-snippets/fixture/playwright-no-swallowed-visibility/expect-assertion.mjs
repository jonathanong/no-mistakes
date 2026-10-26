export async function shown(locator) {
  await expect(locator).toBeVisible();
}

// toBeVisible is an assertion, not a state method, even inside try/catch.
export async function stillShown(locator) {
  try {
    return await expect(locator).toBeVisible();
  } catch {
    return false;
  }
}
