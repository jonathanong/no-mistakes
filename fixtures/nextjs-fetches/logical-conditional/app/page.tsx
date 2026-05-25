export default async function Page() {
  const a = true && fetch('/api/and-right');
  const b = false || fetch('/api/or-right');
  const c = null ?? fetch('/api/nullish-right');

  fetch('/api/unconditional');

  return <div>Page</div>;
}
