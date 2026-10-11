type MutableStatement = { append(text: string): unknown };

export type TxExecutor = (statement: MutableStatement, args?: unknown) => void;

// This same-named factory lives outside the configured executor package.
export function openTransaction(): TxExecutor {
  return (statement) => { statement.append(" OFFSET 1"); };
}
