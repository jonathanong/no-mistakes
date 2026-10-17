export async function afterComputedGate() {
  const query = '/* effects/computed */ SELECT id FROM posts'
  const gate = { [await Promise.resolve('ready')]: true }
  void gate
  return query
}
