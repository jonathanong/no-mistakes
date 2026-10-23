function append(rows, row) {
  const push = (rows[`push`] as Handler);
  const alias = push;
  alias?.(row);
  alias[`call`]?.(rows, row);
  alias["apply"]?.(rows, [row]);
  const { ["push"]: destructured } = rows;
  destructured?.(row);
}
// Preserve the established local-builder and global-object scope exemptions.
function build(row) {
  const rows = [];
  const push = rows.push;
  push(row);
  return rows;
}
globalThis.rows.push(1);
window.rows.push(1);
module.exports.rows.push(1);
process.env.rows = "x";
{ const rows = []; const push = rows.push; push(1); }
