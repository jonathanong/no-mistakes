// The inner catch returns null. The outer boolean is not the visibility failure.
export async function nestedNull(locator) {
  try {
    try {
      return await locator.isVisible();
    } catch {
      return null;
    }
  } catch {
    return false;
  }
}

// This await is in the inner catch, so its rejection reaches the outer catch.
export async function stateInInnerCatch(locator) {
  try {
    try {
      return await locator.click();
    } catch {
      return await locator.isHidden();
    }
  } catch {
    return false;
  }
}

// A nested try without a handler still propagates to the outer catch.
export async function finallyPropagates(locator) {
  try {
    try {
      return await locator.isEnabled();
    } finally {
      locator.ready = true;
    }
  } catch {
    return true;
  }
}

// The binding's await sits in the inner try, so the outer boolean is not evidence.
export async function boundInner(locator) {
  try {
    try {
      const visible = await locator.isVisible();
      return visible;
    } catch {
      return null;
    }
  } catch {
    return false;
  }
}

// An inner catch that rethrows is not proven to reach the outer boolean.
export async function innerRethrow(locator) {
  try {
    try {
      return await locator.isChecked();
    } catch (error) {
      throw error;
    }
  } catch {
    return false;
  }
}
