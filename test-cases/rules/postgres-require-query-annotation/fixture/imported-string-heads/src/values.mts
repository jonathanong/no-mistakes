export const named = `/* importedNamed */ SELECT 1`
export const interpolated = `/* importedInterpolated */ SELECT ${String(1)}`
export const missing = `SELECT 1`
export const partial = `/* ${String(1)} */ SELECT 1`
export let mutable = `/* mutable */ SELECT 1`
mutable = `SELECT 1`
export { named as aliased }
