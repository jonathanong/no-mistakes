export function Page({ type }) {
  return (
    <>
      <script type={type} />
      <script type={`application/ld+json`} />
      <script type="application/json" />
    </>
  );
}
