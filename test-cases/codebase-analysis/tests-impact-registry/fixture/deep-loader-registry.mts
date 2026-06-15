// A registry-shaped file whose dynamic import is buried in an uninvoked nested
// function. Reachability prunes it, so no registry hint should be emitted.
export const registry = {
  load: () => {
    const debug = () => import('./feature.mts');
    return null;
  },
};
