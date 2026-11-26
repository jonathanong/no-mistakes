import type {
  PostgresBoundedStatementKind,
  PostgresBoundedStatementsOptions,
} from "no-mistakes";

const statements: PostgresBoundedStatementKind[] = ["select", "update", "delete"];
const options: PostgresBoundedStatementsOptions = {
  schemaCatalogPath: "catalog.json",
  statements,
};

// @ts-expect-error INSERT is outside the bounded-statements selector.
const invalidStatement: PostgresBoundedStatementKind = "insert";

void [options, invalidStatement];
