async function mutate() {
  const statements = await import('./barrel')
  statements.fragment.text = process.env.SQL
}
