export default async function Fetcher() {
  const response = await fetch("/api/visible");
  return <div>{response.status}</div>;
}
