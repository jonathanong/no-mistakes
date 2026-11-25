export function Valid(dynamicUrl: string, other: { fetch: (u: string) => void }, props: Record<string, string>) {
  void fetch("https://api.example.com/v1/items");
  void window.fetch("https://api.example.com/v1/window");
  void self.fetch("https://api.example.com/v1/self");
  void other.fetch("https://unlisted-custom.example.net/not-global");
  void window?.fetch("https://unlisted-opt.example.net/optional");
  void new WebSocket("wss://rt.example.com:8443/stream");
  void new globalThis.EventSource("https://api.example.com/events");
  void fetch(dynamicUrl);
  void fetch("/api/local");
  const note = `not a policy ${dynamicUrl}`;
  void note;
  return (
    <div>
      <script src="https://scripts.example.com/app.js" />
      <link href="https://unlisted-no-rel.example.net/ignored" />
      <link rel="stylesheet" href="https://styles.example.com/app.css" />
      <link rel="modulepreload" href="https://scripts.example.com/chunk.js" />
      <link rel="icon" href="https://cdn.example.com/favicon.ico" />
      <link rel="preload" as="font" href="https://fonts.example.com/inter.woff2" />
      <link rel="preload" as="script" href="https://scripts.example.com/p.js" />
      <link rel="preload" as="style" href="https://styles.example.com/p.css" />
      <link rel="preload" as="image" href="https://cdn.example.com/p.png" />
      <link rel="preload" as="audio" href="https://app.example.com/p.mp3" />
      <link rel="preload" as="fetch" href="https://api.example.com/p.json" />
      <link rel="preload" as="other" href="https://app.example.com/p.bin" />
      <link rel="help" href="https://app.example.com/help" />
      <link rel="preconnect" href="https://unlisted-hint.example.net" />
      <img {...props} data-kind="logo" src="https://cdn.example.com/logo.png" alt="" />
      <img src={dynamicUrl ? "https://cdn.example.com/a.png" : "https://cdn.example.com/b.png"} alt="" />
      <img src={"https://cdn.example.com/expr.png"} alt="" />
      <img src={`https://cdn.example.com/tpl.png`} alt="" />
      <img src={`https://unlisted.example.net/${dynamicUrl}`} alt="" />
      <img src="https://eu.assets.example.org/banner.webp" alt="" />
      <img src="https://app.example.com/brand.svg" alt="" />
      <img src="/local/avatar.png" alt="" />
      <img src="data:image/png;base64,AAAA" alt="" />
      <iframe src="https://embeds.example.com/player" title="player" />
    </div>
  );
}
