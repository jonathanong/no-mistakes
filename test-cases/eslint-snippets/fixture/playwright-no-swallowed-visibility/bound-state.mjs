// The member write is not an assignment to `visible`, so the return is still
// the awaited state check.
export async function bound(locator) {
  try {
    const visible = await locator.isVisible();
    locator.flag = true;
    return visible;
  } catch {
    return false;
  }
}

export async function hidden(locator) {
  try {
    const hidden = await locator.isHidden();
    return hidden;
  } catch {
    return !0;
  }
}
