export declare function query(sql: unknown): unknown;
export declare function sql(parts: TemplateStringsArray, ...values: unknown[]): {
  append(sql: unknown): unknown;
};
export interface SQLStatement { append(value: unknown): SQLStatement; }
export default sql;
