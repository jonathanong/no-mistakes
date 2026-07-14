export function Page() {
  return (
    <main>
      <button data-pw="aside-button">Aside</button>
      <button data-pw="default-button">Default</button>
      <button data-pw="namespace-button">Namespace</button>
      <button data-pw="namespace-native-name">Namespace native name</button>
      <button data-pw="package-import-button">Package import</button>
      <button data-pw="workspace-export-button">Workspace export</button>
      <button data-pw="ambiguous-button">Ambiguous wrapper declaration</button>
      <button data-pw="ambiguous-namespace-button">Ambiguous namespace declaration</button>
      <button data-pw="recognized-missing-button">Recognized missing target</button>
      <button data-pw="shadowed-button">Shadowed</button>
      <button data-pw="unconfigured-button">Unconfigured</button>
      <button data-pw="mode">Configured non-selector argument</button>
    </main>
  )
}
