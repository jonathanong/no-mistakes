export function sql(strings: TemplateStringsArray, ...values: unknown[]) {
  return { strings, values };
}

export function query(statement: unknown) {
  return statement;
}
