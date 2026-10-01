expect(script).toContain('brew install postgresql@18')
expect(script).toContain('brew link --force libpq@17 openssl@3.5')
expect(script).toContain('brew install --cask temurin@21')
expect(script).toContain('brew services start postgresql@18')
expect(script).toContain('HOMEBREW_PREFIX=$(brew --prefix postgresql@18)')
// A tap-qualified formula is one finding, not also an action ref.
expect(script).toContain('brew install homebrew/core/postgresql@18')
// A slashless `uses:` value is not an action ref, so it stays a formula.
expect(output).toContain('Homebrew uses: postgresql@18')
// A key that only ends in `uses` is not the `uses:` key, so this is a formula.
expect(script).toContain('brew package.uses: homebrew/core/postgresql@18')
// An identifier prefix such as `$` does not make a standalone `uses:` key either.
expect(script).toContain('brew $uses: homebrew/core/postgresql@18')
