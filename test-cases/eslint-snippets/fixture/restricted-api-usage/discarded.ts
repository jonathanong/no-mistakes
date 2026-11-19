declare const page: { goto(path: string): Promise<string> };
declare const sink: (value: unknown) => void;
declare let slot: unknown;

export async function pages() {
  await page.goto("/guide");
  page.goto("/guide");
  void page.goto("/guide");
  (page.goto("/a"), page.goto("/b"));
  (page.goto("/guide"), 1);
  (await page.goto("/guide")) as string;
  page?.goto("/guide");
  const response = await page.goto("/guide");
  slot = await page.goto("/guide");
  sink(page.goto("/guide"));
  page.goto("/guide").then(sink);
  slot = (1, page.goto("/guide"));
  void 0;
  return page.goto("/guide");
}

export const lazy = () => page.goto("/guide");
page.goto("/guide");
