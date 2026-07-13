// Intentionally ignored: every check phase must use the request snapshot.
export default function Ignored() {
  return (
    <main>
      <button data-testid="ignored-only">Ignored</button>
      <button data-testid="ignored-only">Ignored duplicate</button>
    </main>
  );
}
