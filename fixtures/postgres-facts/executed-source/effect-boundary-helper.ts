export function appendLiteral(query: { append: (suffix: string) => void }) {
  query.append(' WHERE id = 1')
}

export async function appendAfterGate(query: { append: (suffix: string) => void }) {
  await Promise.resolve('ready')
  query.append(' UPDATE posts SET title = title')
}
