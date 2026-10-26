const TEST_ID = 'selector-control';

export function DefaultId({ id = 'literal-default' }: { id?: string }) {
  return <input id={id} />;
}

export default function Page() {
  return (
    <main>
      <h1 id="literal-unique">Unique literal</h1>
      {/* Dynamic test IDs are outside this HTML-id rule's low diagnostics. */}
      <button data-testid={TEST_ID}>Control</button>
    </main>
  );
}
