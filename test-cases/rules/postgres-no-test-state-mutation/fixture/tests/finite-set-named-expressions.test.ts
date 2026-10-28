import { query, sql } from '@example/db';
// The expression's own Set name is visible inside its body, including methods.
export const ClassCase = class Set {
  constructor() { return { has: () => true }; }
  static run(table: string) {
    const allowed = new Set(['safe_one']);
    if (!allowed.has(table)) throw new Error('unsafe');
    return query(sql`/* namedClass */ SELECT 1 FROM `.append(table));
  }
};
