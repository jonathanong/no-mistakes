declare const page: { goto(path: string): Promise<string> };
declare const cond: boolean;
declare const other: unknown;
declare const sink: (value: unknown) => void;
declare let slot: unknown;

export function discarded() {
  other ?? page.goto("/a");
  other || page.goto("/b");
  cond && page.goto("/c");
  cond ? page.goto("/d") : page.goto("/e");
  cond ? other : page.goto("/f");
  other || (cond && page.goto("/g"));
  cond && (page.goto("/h"), other);
  cond && void page.goto("/i");
  for (page.goto("/j"); cond; page.goto("/k")) sink(0);
  for (page.goto("/l"), other; ; other, page.goto("/m")) break;
  cond ? (cond ? page.goto("/n") : other) : other;
}

export function used() {
  const a = page.goto("/a") ?? other;
  slot = cond && page.goto("/b");
  slot = cond ? page.goto("/c") : other;
  sink(cond && page.goto("/d"));
  page.goto("/e") ? other : other;
  for (; page.goto("/f");) break;
  for (const x of [page.goto("/g")]) sink(x);
  for (const key in page.goto("/h")) sink(key);
  if (page.goto("/i")) sink(0);
  while (page.goto("/j")) break;
  return cond && page.goto("/k");
}

export const lazy = () => cond && page.goto("/l");
