export default function sql(text: TemplateStringsArray, ...values: unknown[]) {
  return { text: text.join(""), values };
}
