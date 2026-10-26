// Named handlers are not resolved. This rethrow must stay silent, and so must
// a named function whose body is not read.
async function handleError(error) {
  throw error;
}

export async function checked(locator) {
  return await locator.isVisible().catch(handleError);
}
