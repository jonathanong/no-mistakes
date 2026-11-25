export function Page(id: string, args: [string], dynAs: string, dynHref: string) {
  void fetch(...args);
  void fetch("https://shared.example.com" + "/ok");
  void fetch("https://shared.example.com/" + id);
  void fetch("https://" + id);
  void fetch(`//shared.example.com/${id}`);
  void fetch(`data:text/plain,${id}`);
  return (
    <div>
      <script src="https://shared.example.com/ok.js" />
      <script src="https://dev.example.com/blocked.js" />
      <link rel="preload" as={dynAs} href="https://shared.example.com/p.js" />
      <link rel="preload" as={dynAs} href={dynHref} />
      <link rel="preload" as={dynAs} href="/local.js" />
    </div>
  );
}
