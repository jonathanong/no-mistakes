export default async function Fetcher() {
  // The next-line directive wins when both supported directives cover one finding.
  // no-mistakes-disable-next-line assert-no-fetch: next-line is authoritative
  await fetch('/api/first'); // no-mistakes-disable-line assert-no-fetch: same-line also matches
  return <div />;
}
