export default async function Page() {
  try {
    fetch('/api/handled');
  } catch (e) {
    console.error(e);
  }

  try {
    fetch('/api/finally-only');
  } finally {
    console.log('done');
  }

  fetch('/api/unhandled');

  return <div>Page</div>;
}
