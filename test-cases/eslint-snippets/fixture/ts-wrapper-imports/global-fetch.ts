const { ["fetch" as const]: fetchAlias } = globalThis;
let fn: (input: string) => Promise<Response>;

globalThis["fetch" as const]("/a");
fetch["call" as const](null, "/b");
fetchAlias("/c");
((() => {
  fn = fetch;
}) as () => void)();
fn("/d");
