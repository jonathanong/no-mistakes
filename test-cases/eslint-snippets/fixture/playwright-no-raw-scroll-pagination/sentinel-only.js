export async function scrollToSentinel(page) {
  const sentinel = page.locator(".load-more");
  await sentinel.scrollIntoViewIfNeeded();
}
