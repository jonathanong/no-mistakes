/* global sentinel */
export async function scrollUnresolvedSentinels(locators) {
  await sentinel.scrollIntoViewIfNeeded();
  {
    const { sentinel } = locators;
    await sentinel.scrollIntoViewIfNeeded();
  }
  {
    const sentinel = locators.first();
    await sentinel.scrollIntoViewIfNeeded();
  }
}
