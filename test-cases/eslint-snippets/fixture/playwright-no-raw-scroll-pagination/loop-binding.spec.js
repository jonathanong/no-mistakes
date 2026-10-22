page.waitForResponse(() => url.searchParams.has("cursor"));
// A const loop binding has no initializer and cannot identify a global scroll function.
for (const scroll of callbacks) scroll(0, 100);
