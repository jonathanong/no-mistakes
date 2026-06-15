// Outside the configured selectorRoots (app, components) and not a test file,
// so this match must be ignored when roots are configured.
export function Elsewhere() {
  return <div data-pw="search-bar">elsewhere</div>;
}
