// A registry-shaped file whose dynamic import is buried in an uninvoked nested
// function. `import-dynamic` still follows that `import()`, so a registry hint
// is emitted.
export const registry = {
  load: () => {
    const debug = () => import("./feature.mts");
    return null;
  },
};
