declare const page: { goto(path: string): Promise<string> };
declare const handler: (value: unknown) => void;

export function chained() {
  page.goto("/a").catch(handler);
  page.goto("/b").then(handler);
  page.goto("/c").finally(handler);
  page.goto("/d")?.length;
}
