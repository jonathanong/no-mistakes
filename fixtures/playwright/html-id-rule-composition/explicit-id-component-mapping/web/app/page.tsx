function Section({ htmlId }: { htmlId: string }) {
  return <section id={htmlId}>Content</section>;
}

export default function Page() {
  return <Section htmlId="explicit-component-id" />;
}
