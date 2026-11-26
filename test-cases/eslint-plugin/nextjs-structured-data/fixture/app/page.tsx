export function Page() {
  return (
    <>
      <script
        id="page-schema"
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: "{}" }}
      />
      <script
        id="secondary-schema"
        type={"application/ld+json"}
        dangerouslySetInnerHTML={{ __html: "{}" }}
      />
    </>
  );
}
