import {
  extractExecutedPostgresSql,
  type ExecutedPostgresSql,
  type ExecutedPostgresSqlKind,
  type ExecutedPostgresSqlOptions,
  type ExecutedPostgresTrustedSqlTag,
} from "no-mistakes";

const trustedTag: ExecutedPostgresTrustedSqlTag = { module: "@db", name: "sql" };
const options: ExecutedPostgresSqlOptions = {
  sources: [{ fileName: "query.ts", source: "query('SELECT 1')" }],
  importSpecifier: "@db",
  trustedSqlTags: [trustedTag],
};
const result: Promise<ExecutedPostgresSql[]> = extractExecutedPostgresSql(options);

result.then((queries) => {
  const kind: ExecutedPostgresSqlKind = queries[0]!.kind;
  void kind;
});
