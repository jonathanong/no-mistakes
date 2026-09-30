# Call workspace packages fixture

This fixture covers call-target resolution through workspace package names.
`src/entry.ts` calls functions imported from `@scope/lib`, from its `exports`
subpaths (`@scope/lib/util`, `@scope/lib/barrel`, `@scope/lib/data`), and from a
namespace import of a subpath. `src/reexport.ts` re-exports a workspace function
under a new name so a relative import reaches it through the package specifier.

It also locks the boundaries. A bare external specifier (`react`) and a package
name with no workspace package (`@scope/missing`) stay unresolved module
exports, while an export that resolves to a non-callable value in a workspace
package becomes an unknown call, the same as it does through a relative import.
