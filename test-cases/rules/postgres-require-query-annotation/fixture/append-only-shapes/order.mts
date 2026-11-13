const ORDER = ['id', 'name']
export function addOrder(q) {
  q.append('(')
  ORDER.forEach((col, i) => {
    if (i > 0) q.append(', ')
    q.append(`${col}`)
  })
  return q.append(')')
}
