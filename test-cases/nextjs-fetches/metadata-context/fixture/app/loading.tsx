export default function Loading() {
  fetch('/api/loading-data');
  return <div>Loading...</div>;
}
