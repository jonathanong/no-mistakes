export default async function Widgets() {
  const cacheMode = "force-cache";

  await fetch("/widgets", { cache: cacheMode });
  return <main>Widgets</main>;
}
