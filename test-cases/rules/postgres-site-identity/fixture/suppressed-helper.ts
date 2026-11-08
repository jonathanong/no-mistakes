export function statementSql() {
  // no-mistakes-disable-next-line postgres-bounded-statements -- helper exemption
  return 'SELECT id FROM accounts';
}
