// Aliases share one stub outside src so consumer scopes select only SQL hosts.
export function query(_text: string): void {}
export default function sql(strings: TemplateStringsArray): string {
  return strings[0];
}
