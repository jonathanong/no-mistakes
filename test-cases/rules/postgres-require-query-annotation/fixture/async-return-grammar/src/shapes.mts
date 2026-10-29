import sql from 'sql-template-strings'

async function setupValuesBeforeRoot() {
  let seed = 1
  var other = 2
  const left = 3, right = 4
  log(seed, other, left, right)
  const query = sql`/* setup */ SELECT 1`
  return query
}
async function wrongTerminalStatement() {
  const query = sql`/* terminal */ SELECT 1`
  query.append(' LIMIT 1')
}
async function nonIdentifierTag() {
  const query = db.sql`/* member tag */ SELECT 1`
  return query
}
async function nonNativeRoot() {
  const query = '/* primitive */ SELECT 1'
  return query
}
async function emptyAppendRoot() {
  const query = sql`/* empty append */ SELECT 1`.append()
  return query
}
async function emptyAppendStatement() {
  const query = sql`/* empty statement append */ SELECT 1`
  query.append()
  return query
}
async function rootDepthEight() {
  const query = sql`/* root depth eight */ SELECT 1`.append('1').append('2').append('3').append('4').append('5').append('6').append('7').append('8')
  return query
}
async function rootDepthNine() {
  // Nine receivers must not extend the bounded native-root grammar.
  const query = sql`/* root depth nine */ SELECT 1`.append('1').append('2').append('3').append('4').append('5').append('6').append('7').append('8').append('9')
  return query
}
async function discardedDepthEight() {
  const query = sql`/* discarded depth eight */ SELECT 1`
  query.append('1').append('2').append('3').append('4').append('5').append('6').append('7').append('8')
  return query
}
async function discardedDepthNine() {
  const query = sql`/* discarded depth nine */ SELECT 1`
  // The ninth receiver must fail closed rather than truncate the chain.
  query.append('1').append('2').append('3').append('4').append('5').append('6').append('7').append('8').append('9')
  return query
}
async function bareRootExpression() {
  const query = sql`/* bare root */ SELECT 1`
  query
  return query
}
async function callerAccepted() {
  const query = await factory()
  read(query)
}
async function callerNonCallTerminal() {
  const query = await factory()
  query
}
async function callerNonIdentifierArgument() {
  const query = await factory()
  read('SELECT 1')
}
async function callerDifferentArgumentOwner() {
  const query = await factory()
  read(otherQuery)
}
