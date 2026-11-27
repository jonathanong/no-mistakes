import type {
  PostgresBoundedStatementsOptions,
  PostgresLegacyUnanalyzableSql,
  PostgresNoMutableSqlRegistryOptions,
  PostgresNoUnboundedSqlIdentifierOptions,
} from "no-mistakes";

const bounded: PostgresBoundedStatementsOptions = {
  schemaCatalogPath: "catalog.json",
  unanalyzableSql: "fail",
};
const registry: PostgresNoMutableSqlRegistryOptions = { unanalyzableSql: "fail" };
const identifier: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzableSql: "fail" };
const modern: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzable: "report" };
const legacy: PostgresLegacyUnanalyzableSql[] = ["fail", "advise", "ignore"];

// @ts-expect-error The modern policy spells this "report".
const invalidModern: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzable: "fail" };
// @ts-expect-error The legacy policy spells this "fail".
const invalidLegacy: PostgresBoundedStatementsOptions = { ...bounded, unanalyzableSql: "report" };

void [registry, identifier, modern, legacy, invalidModern, invalidLegacy];
