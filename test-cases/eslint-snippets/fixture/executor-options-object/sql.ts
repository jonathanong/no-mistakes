export interface SqlStatement {
  text: string;
  values: readonly unknown[];
}
declare function sql(strings: TemplateStringsArray, ...values: unknown[]): SqlStatement;
export default sql;
