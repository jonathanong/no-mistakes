// export type * is skipped (line 17) and export * as X is skipped (line 17)
// first export * from empty source returns None (line 24)
// second and third export * both find the symbol - triggers ambiguity (line 27)
export type * from './vitest.object-star-export-source'
export * as ns from './vitest.object-star-export-source'
export * from './vitest.object-star-export-empty'
export * from './vitest.object-star-export-source'
export * from './vitest.object-star-export-source2'
