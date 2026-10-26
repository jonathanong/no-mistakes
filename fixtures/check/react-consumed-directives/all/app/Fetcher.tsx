export default async function Fetcher() {
  // Both comments hide independent fetch contributions to one component diagnostic.
  // no-mistakes-disable-next-line assert-no-fetch: first intentional request
  await fetch('/first');
  await fetch('/second'); // no-mistakes-disable-line assert-no-fetch: second intentional request
  return <div />;
}
