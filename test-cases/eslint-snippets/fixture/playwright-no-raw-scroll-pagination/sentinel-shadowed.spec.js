export async function scrollToSentinel(page) {
  const sentinel = page.locator(".load-more");
  await sentinel.isVisible();

  async function inspect(sentinel) {
    await sentinel.scrollIntoViewIfNeeded();
  }
  await inspect(page.getByText("Details"));
}
