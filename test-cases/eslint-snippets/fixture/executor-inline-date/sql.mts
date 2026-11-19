export interface Statement {
  text: string;
  values: unknown[];
  append(other: Statement | string): Statement;
}
declare function sql(strings: TemplateStringsArray, ...values: unknown[]): Statement;
export default sql;
