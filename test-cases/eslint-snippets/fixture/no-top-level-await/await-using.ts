await using handle = openHandle();
if (enabled) {
  await using scoped = openHandle();
}
using syncHandle = openSyncHandle();
export async function initialize() {
  await using local = openHandle();
}
if (import.meta.main) {
  await using guarded = openHandle();
}
// no-mistakes-disable-next-line no-top-level-await -- dedicated runtime entrypoint
await using suppressed = openHandle();
