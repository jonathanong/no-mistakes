with (globalThis) {
  it.each(['not provably bound'])('rejected', field => { query(field) })
}
