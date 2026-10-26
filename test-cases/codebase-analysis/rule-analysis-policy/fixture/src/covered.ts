// The test-only adopter treats helper-based input as unanalyzable at low effort.
export const literal = 'SELECT 1';
export function helper() { return literal; }
export const indirect = helper();
