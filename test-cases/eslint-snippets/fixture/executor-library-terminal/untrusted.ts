import type { SqlStatement } from "./sql";

export declare function rawSql(strings: TemplateStringsArray, ...values: unknown[]): SqlStatement;
