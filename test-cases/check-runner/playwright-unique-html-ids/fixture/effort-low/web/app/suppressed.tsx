// Low-only uncertainty diagnostics must honor the same directives as findings.
export function SuppressedNext({ id }: { id: string }) {
  // no-mistakes-disable-next-line playwright-unique-html-ids
  return <section id={id} />;
}

export function SuppressedLine({ id }: { id: string }) {
  return <aside id={id} />; // no-mistakes-disable-line playwright-unique-html-ids
}
