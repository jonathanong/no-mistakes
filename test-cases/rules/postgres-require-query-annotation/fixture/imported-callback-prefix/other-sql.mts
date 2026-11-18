export class SqlStatement {
  constructor(readonly text: string) {}
}

export function sql(parts: TemplateStringsArray, ...values: unknown[]): SqlStatement {
  let text = parts[0];
  for (let index = 0; index < values.length; index++) {
    const value = values[index];
    text += value instanceof SqlStatement ? value.text : '$' + (index + 1);
    text += parts[index + 1];
  }
  return new SqlStatement(text);
}
