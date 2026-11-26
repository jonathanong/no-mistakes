export function Page(dynamicUrl: string, host: string, id: string, dynamicRel: string, spreadProps: Record<string, string>) {
  void fetch("http://[::1]:3000/health");
  void fetch("https://[2001:db8::2]:8443/api");
  void fetch(`/api/users/${id}`);
  void fetch(dynamicUrl);
  void fetch(`https://${host}/v1`);
  return (
    <div>
      <img src="//cdn.example.com/ok.png" alt="" />
      <img src="https://[2001:0db8::1]/v6.png" alt="" />
      <img src={`https://cdn.example.com/u/${id}.png`} alt="" />
      <script src="//unlisted-proto.example.net/sdk.js" />
      <img src={dynamicUrl} alt="" />
      <img src="https://bad host/x.png" alt="" />
      <link rel={dynamicRel} href="https://cdn.example.com/a.css" />
      <img {...spreadProps} alt="" />
      <script {...spreadProps} />
      <iframe src="https://embeds.example.com/ok" {...spreadProps} title="player" />
      <link {...spreadProps} />
      <img {...spreadProps} src="https://cdn.example.com/override.png" alt="" />
    </div>
  );
}
