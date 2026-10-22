// An unrelated parameter must not enable the file's cursor-pagination gate.
async function helper(waitForResponse) {
  await waitForResponse(() => url.searchParams.has("cursor"));
  const wait = waitForResponse;
  await wait(() => url.searchParams.has("cursor"));
}
window.scrollTo(0, 100);
function renamedParameter(callback) {
  const waitForResponse = callback;
  waitForResponse(() => url.searchParams.has("cursor"));
}
