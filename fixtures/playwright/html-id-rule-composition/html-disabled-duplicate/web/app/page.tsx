export default function Page() {
  return (
    <main>
      {/* Uniqueness intentionally scans IDs even when coverage does not. */}
      <button id="duplicate-disabled">Save</button>
      <section id="duplicate-disabled">Duplicate</section>
    </main>
  );
}
