// A bare module type carries no member name, so it is conservatively unknown.
export type Compound = typeof import('./source.mjs');
