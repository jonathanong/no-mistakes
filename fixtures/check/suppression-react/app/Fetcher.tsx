// no-mistakes-disable-file assert-no-fetch: intentional fixture fetch
export default async function Fetcher() {
  await fetch('/api/users');
  return <div />;
}
