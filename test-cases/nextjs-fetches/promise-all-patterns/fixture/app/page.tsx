export default async function Page() {
  const settled = await Promise.allSettled([
    fetch('/api/settled-1'),
    fetch('/api/settled-2'),
  ]);

  fetch('/api/sequential');

  return <div>Page</div>;
}
