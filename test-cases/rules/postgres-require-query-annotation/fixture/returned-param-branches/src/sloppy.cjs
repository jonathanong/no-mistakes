const sql = require('sql-template-strings')
// A sloppy CommonJS function cannot establish the existing imported native tag proof.
function factory(select, extra, from = sql` FROM records r`) {
  const statement = select.append('id')
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}
module.exports.factory = factory
