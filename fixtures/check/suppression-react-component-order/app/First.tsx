export default async function First() {
  // no-mistakes-disable-next-line assert-no-fetch: this component is intentionally suppressed
  await fetch('/api/first');
  return <span />;
}
