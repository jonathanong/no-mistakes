import {
  check,
  type PostgresJsonRelationIdsAllowEntry,
  type PostgresJsonRelationIdsOptions,
} from "no-mistakes";

const historyAllowance: PostgresJsonRelationIdsAllowEntry = {
  object: "table:invoice_revisions",
  reason: "History snapshots retain identifiers for audit display only.",
};

const ruleOptions: PostgresJsonRelationIdsOptions = {
  schemaCatalogPath: "db/schema.json",
  idKeyPattern: "(^id$|Id$|_id$)",
  keyColumnMap: { accountId: "account_id" },
  fingerprintNamePattern: "",
  allow: [historyAllowance],
  include: ["src/**/*.ts"],
  exclude: ["src/generated/**"],
  sqlInclude: ["db/migrations/**/*.sql"],
  importSpecifier: "@example/db",
  executorNames: ["query"],
  executorFactoryNames: ["createDatabase"],
  executorTypeNames: ["Database"],
  trustedSqlTags: [{ module: "@example/sql", name: "sql" }],
};

const configuredRule = {
  rule: "postgres-json-relation-ids",
  scope: "repository",
  options: ruleOptions,
} as const;

const report = check({ root: ".", config: ".no-mistakes.yml" });
void [configuredRule, report];
