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
// The whole `owner/repo/path` is the ref, not its last two components.
expect(workflow).toContain('uses: github/codeql-action/init@v3')
expect(workflow).toContain('uses: octo-org/example-repo/.github/workflows/reusable.yml@v1')
expect(workflow).toContain('uses: github/codeql-action/upload-sarif@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v3.25.0')
// Zero versions are read in the last two path components and the ref, so a `v0.0.0`
// directory above them is a path, not a placeholder.
expect(workflow).toContain('uses: owner/repo/v0.0.0/path/action@v1')
expect(workflow).toContain('uses: owner/repo/v1.0.0/x/y@v1')
// A port stays with its host, and a component with no name in it ends the path.
expect(url).toContain('https://registry.npmjs.org:443/o/r@v1')
expect(workflow).toContain('uses: owner/repo/../path/action@v1')
