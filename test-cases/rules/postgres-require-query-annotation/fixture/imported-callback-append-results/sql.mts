export class SqlStatement {
  constructor(public text: string) {}
  append(tail: SqlStatement): this { this.text += tail.text; return this; }
}
export function sql(parts: TemplateStringsArray, ...values: unknown[]): SqlStatement {
  return new SqlStatement(parts.join('?'));
}
