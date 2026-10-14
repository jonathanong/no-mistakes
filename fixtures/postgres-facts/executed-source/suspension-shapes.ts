const query = '/* suspension/shapes */ SELECT id FROM posts'

export async function baseline() {
  const gate = [1]
  void gate
  return query
}

export async function spreadGate() {
  const gate = Math.max(...[await Promise.resolve(1)])
  void gate
  return query
}

export async function indexGate() {
  const gate = (await Promise.resolve([1]))[0]
  void gate
  return query
}

export async function templateGate() {
  const gate = 'x' + await Promise.resolve('y')
  void gate
  return query
}

export async function childrenGate() {
  const gate = (await Promise.resolve(1)) > 0
  void gate
  return query
}

export async function containerGate() {
  const gate = [await Promise.resolve(1)]
  void gate
  return query
}

export async function opaqueGate() {
  const gate = +(await Promise.resolve(1))
  void gate
  return query
}

export async function writeGate() {
  const gate = { value: 0 }
  gate.value = await Promise.resolve(1)
  return query
}
