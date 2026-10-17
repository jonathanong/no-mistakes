export function appendLiteral(query: { append: (suffix: string) => void }) {
  query.append(' WHERE id = 1')
}

export function appendConcatenated(query: { append: (suffix: string) => void }) {
  query.append(' ORDER BY ' + 'id')
}

export async function appendAfterGate(query: { append: (suffix: string) => void }) {
  await Promise.resolve('ready')
  query.append(' UPDATE posts SET title = title')
}
