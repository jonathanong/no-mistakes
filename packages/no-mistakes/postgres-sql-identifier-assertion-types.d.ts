/** An imported assertion that validates an identifier against a finite allowlist. */
export interface PostgresSqlIdentifierAssertion {
  module: string;
  name: string;
  /** Zero-based position of the allowlist argument. */
  allowlistArgument: number;
}
