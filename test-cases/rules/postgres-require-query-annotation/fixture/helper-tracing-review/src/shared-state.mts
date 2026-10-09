import sql from 'sql-template-strings';
export interface SharedState { text: string }
export type SharedName = string;
const statement = sql`/* shared module builder */ SELECT 1`;
export function sharedStatement() { return statement; }
export function mutateSharedStatement() { statement.text = 'SELECT 2'; }

const appendedStatement = sql`/* shared append */ `;
export function appendSharedStatement() { appendedStatement.append('SELECT 1'); return appendedStatement; }
