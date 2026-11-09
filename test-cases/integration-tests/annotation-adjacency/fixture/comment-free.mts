export function first() { return 1; }
export const second = () => first();
export function third() { return second(); }
