export default async function Child() {
  // no-mistakes-disable-next-line assert-no-fetch: this child call is intentional
  await fetch('/data/child');
  return <span />;
}
