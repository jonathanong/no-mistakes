export function placeholders(index: number) {
  const valid = [`$${index + 1}`]
  const invalid = [`$${true}`]
  const validJoined = valid.join(', ')
  const invalidJoined = invalid.join(', ')
  return { validJoined, invalidJoined }
}
