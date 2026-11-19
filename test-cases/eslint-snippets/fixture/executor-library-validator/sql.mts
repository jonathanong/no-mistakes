export interface Statement {
  text: string;
}
declare function sql(strings: TemplateStringsArray, ...values: unknown[]): Statement;
export default sql;
