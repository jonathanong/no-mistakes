// Sync rewrite of async-builder.ts: `async` is removed and the same literal
// SQL is returned. The executed-source projection must judge the caller
// complete, not unanalyzable.
export function buildSql() {
  return '/* async/read */ SELECT id FROM posts'
}
