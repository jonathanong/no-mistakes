export async function quiet(locator, name) {
  await locator.isVisible().catch(() => null);
  await locator.isVisible().catch(() => 0);
  await locator.isVisible().catch(() => locator);
  await locator.isVisible().catch(() => {
    return;
  });
  await locator.isVisible().catch();
  await locator.click().catch(() => false);
  await locator.isVisible.catch(() => false);
  await locator[name]().catch(() => false);
  // The nested return is not the catch callback's own result.
  await locator.isVisible().catch(() => {
    const hide = () => {
      return false;
    };
    throw hide();
  });
  try {
    return await locator.click();
  } catch {
    return false;
  }
}

export async function withoutAwait(locator) {
  try {
    return locator.isVisible();
  } catch {
    return false;
  }
}

export async function catchNull(locator) {
  try {
    return await locator.isVisible();
  } catch {
    return null;
  }
}

export async function finallyOnly(locator) {
  try {
    return await locator.isVisible();
  } finally {
    locator.ok = true;
  }
}

export async function nestedRead(locator) {
  try {
    const read = async () => {
      return await locator.isVisible();
    };
    return await read();
  } catch {
    return false;
  }
}

export function bareReturn(locator) {
  try {
    return;
  } catch {
    return false;
  }
  return locator;
}
