export default async function Child() {
  // Both parents inherit this one suppressed fetch. The audit must retain
  // one record per parent even though the source location is shared.
  // no-mistakes-disable-next-line assert-no-fetch: shared child fetch is intentional
  await fetch('/data/child');
  return <span />;
}
