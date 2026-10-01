expect(workflow).toContain('uses: actions/checkout@v6.0.2')
expect(sha).toContain('actions/setup-node@de0fac2e4500dabe0009e67214ff5f5447ce83dd')
// A `uses:` value on a Homebrew line is an action ref, not a formula: the ref has the
// formula shape and the line says Homebrew, but only the action ref is a workflow pin.
expect(workflow).toContain('uses: Homebrew/actions/setup-homebrew@4')
expect(workflow).toContain('uses: Homebrew/brew@4.1')
expect(workflow).toContain('"uses": "Homebrew/actions/setup-homebrew@4"')
// A flow mapping with a nested flow value before `uses` is still a workflow key.
expect(workflow).toContain('{ env: { HOMEBREW: yes }, uses: Homebrew/actions/setup-homebrew@4 }')
// A quoted closer before `uses` does not end the flow mapping.
expect(workflow).toContain('{ name: "}", uses: Homebrew/actions/setup-homebrew@4 }')
