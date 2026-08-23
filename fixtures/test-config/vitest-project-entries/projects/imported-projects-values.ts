export const importedProjects = [
  './imported-allowed/vitest.config.ts',
  './imported-excluded/vitest.config.ts',
  // Imported exclusions are global across the flattened project array.
  '!./negated-outer.config.ts',
]

export type TypeOnlyProjects = string[]
