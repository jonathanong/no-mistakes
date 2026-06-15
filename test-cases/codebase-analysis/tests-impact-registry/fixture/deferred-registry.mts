// A registry defined first and exported later (`const … ; export { … }`). Its
// dynamic-import map must still be reachable so the hint fires.
const registry = { feature: () => import('./feature.mts') };

export { registry };
