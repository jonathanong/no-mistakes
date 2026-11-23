import type {
  PostgresSqlStatementPolicyClause,
  PostgresSqlStatementPolicyOptions,
} from "no-mistakes";

const clauses: PostgresSqlStatementPolicyClause[] = ["create-extension-version", "drop-cascade"];
const options: PostgresSqlStatementPolicyOptions = {
  bannedClauses: clauses,
  bannedStatements: [],
  sqlInclude: ["db/migrations/**/*.sql"],
};

// @ts-expect-error bannedClauses accepts only the supported clause names.
const invalid: PostgresSqlStatementPolicyOptions = { bannedClauses: ["unknown-clause"] };

void [options, invalid];
