export default function sql(parts: TemplateStringsArray) {
  return {
    text: parts.join(""),
    append(text: string) {
      this.text += text;
      return this;
    },
  };
}
