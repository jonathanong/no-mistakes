function sql(text: TemplateStringsArray) {
  return text;
}

async function localTag() {
  return sql`SELECT 1`;
}
