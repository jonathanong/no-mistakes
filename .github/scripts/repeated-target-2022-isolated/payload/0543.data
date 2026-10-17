export function runOrdered(spec: {
  statement: (ctes: string) => string
  readRows: (statement: string) => unknown
}) {
  const ctes = 'input_data AS (SELECT id FROM posts)'
  return spec.readRows(spec.statement(ctes))
}
