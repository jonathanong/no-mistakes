// Hand-maintained registry: changing a feature here usually means the registry
// entry must be updated too. Registers both features via dynamic imports.
export const registry = {
  feature: () => import('./feature.mts'),
  feature2: () => import('./feature2.mts'),
};
