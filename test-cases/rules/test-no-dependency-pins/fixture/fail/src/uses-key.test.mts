// YAML written inside a JavaScript string. A `\\\"` inside a `\"` scalar is a quote of the
// scalar, not its end, so the brace after it stays quoted and `uses` stays a key.
expect(workflow).toContain("{ name: \"a\\\"}x\", uses: Homebrew/actions/setup-homebrew@4 }")
expect(workflow).toContain('{ name: \'a\\\'}x\', uses: Homebrew/actions/setup-homebrew@4 }')
expect(workflow).toContain("{ name: \"a\\\"]x\", uses: Homebrew/actions/setup-homebrew@4 }")
// Two JavaScript layers write each quote with three backslashes.
expect(workflow).toContain("{ name: \\\"}\\\", uses: Homebrew/actions/setup-homebrew@4 }")
// The scalar ends at its closing quote, so the brace after it closes the mapping and the comma is prose.
expect(output).toContain("brew { a: \"x\\\"{\" }, uses: homebrew/core/postgresql@18")
