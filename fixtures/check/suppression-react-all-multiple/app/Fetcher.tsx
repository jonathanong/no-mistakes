import Child from './Child';

export default async function Fetcher() {
  // no-mistakes-disable-next-line assert-no-fetch: this parent call is intentional
  await fetch('/api/first');
  return <Child />;
}
