import { sentinel } from "./locators";

export async function scrollImportedAndFactorySentinels(page, buildSentinel, feed) {
  await sentinel.scrollIntoViewIfNeeded();
  await sentinel.first().scrollIntoViewIfNeeded();
  {
    const sentinel = buildSentinel(page);
    await sentinel.scrollIntoViewIfNeeded();
  }
  {
    const sentinel = feed.sentinelLocator();
    await sentinel.last().scrollIntoViewIfNeeded();
  }
}
