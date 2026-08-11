export default async function Fetcher() {
  // no-mistakes-disable-next-line assert-no-fetch: intentional fixture fetch
  await fetch('/api/users');
  return <div />;
}
