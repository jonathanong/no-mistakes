function arbitrary(value: unknown) {
  return value
}

export default arbitrary([
  { test: { name: 'must-not-be-a-workspace-project' } },
])
