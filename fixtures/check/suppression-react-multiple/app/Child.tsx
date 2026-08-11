export default async function Child() {
  await fetch('/data/child');
  return <span />;
}
