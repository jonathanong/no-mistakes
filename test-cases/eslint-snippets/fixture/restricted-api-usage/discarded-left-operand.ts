declare const page: { goto(path: string): Promise<string> };
declare const other: unknown;

export function leftOperandIsATest() {
  page.goto("/a") && other;
  page.goto("/b") || other;
  page.goto("/c") ?? other;
  page.goto("/d") ? other : other;
  (page.goto("/e") && other) || other;
  other && (page.goto("/f") ?? other);
  page.goto("/g")?.length && other;
}
