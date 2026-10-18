import { query, read } from '@example/db'

export function shadowImportedQuery() {
  // A CommonJS namespace shadows the callable import, even with the same name.
  const query = require('@example/db')
  query('SELECT 1')
  query.query('SELECT 2')
}

// A callable named export does not prove a `.query` member for destructuring.
const { query: unrelated } = read
unrelated('SELECT 3')
