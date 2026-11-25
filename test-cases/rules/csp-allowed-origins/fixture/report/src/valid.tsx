export function Valid(dynamicUrl: string) {
  void fetch("https://api.example.com/v1/items");
  void new WebSocket("wss://rt.example.com:8443/stream");
  void fetch(dynamicUrl);
  void fetch("/api/local");
  return (
    <div>
      <script src="https://scripts.example.com/app.js" />
      <link rel="stylesheet" href="https://styles.example.com/app.css" />
      <link rel="modulepreload" href="https://scripts.example.com/chunk.js" />
      <link rel="icon" href="https://cdn.example.com/favicon.ico" />
      <link rel="preload" as="font" href="https://fonts.example.com/inter.woff2" />
      <link rel="preconnect" href="https://unlisted-hint.example.net" />
      <img src="https://cdn.example.com/logo.png" alt="" />
      <img src="https://eu.assets.example.org/banner.webp" alt="" />
      <img src="https://app.example.com/brand.svg" alt="" />
      <img src="/local/avatar.png" alt="" />
      <img src="data:image/png;base64,AAAA" alt="" />
      <iframe src="https://embeds.example.com/player" title="player" />
    </div>
  );
}
