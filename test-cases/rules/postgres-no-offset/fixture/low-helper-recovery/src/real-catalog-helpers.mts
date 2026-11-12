import SQL from 'sql-template-strings';
export function make() { return SQL`SELECT 1`; }
export const token = 42;
export type Contract = string;
