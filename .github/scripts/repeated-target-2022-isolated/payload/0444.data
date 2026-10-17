export async function buildAfterObjectGate() {
  const statement = '/* async/object */ SELECT id FROM posts'
  const context = { gate: await Promise.resolve('ready') }
  void context
  return statement
}

export async function buildAfterTaggedGate() {
  const statement = '/* async/tagged */ SELECT id FROM posts'
  const trace = String.raw`gate=${await Promise.resolve('ready')}`
  void trace
  return statement
}

export async function buildAfterCallGate() {
  const statement = '/* async/call */ SELECT id FROM posts'
  const gate = Promise.resolve(await Promise.resolve('ready'))
  void gate
  return statement
}
