export interface Statement {
  text: string;
  append(text: string): Statement;
}
declare function sql(strings: TemplateStringsArray, ...values: unknown[]): Statement;
export default sql;
