export default function sql(strings: TemplateStringsArray, ...values: unknown[]): string {
  return strings.join("?");
}
