class ConstructedError extends Error {}
export function ordinary() { return new ConstructedError(); }
ordinary();
