export default async function HomePage() {
  fetch('/api/unconditional');

  if (true) {
    fetch('/api/conditional');
  }

  try {
    const [a, b] = await Promise.all([
      fetch('/api/parallel-1'),
      fetch('/api/parallel-2'),
    ]);
  } catch (e) {
    console.error(e);
  }

  const result = true ? fetch('/api/ternary-a') : fetch('/api/ternary-b');

  return <div>Home</div>;
}
