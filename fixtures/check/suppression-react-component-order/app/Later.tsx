export default async function Later() {
  await fetch('/api/later');
  return <span />;
}
