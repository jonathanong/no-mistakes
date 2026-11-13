import { read, type QueryExecutor } from '@example/db'
function receive(query: QueryExecutor, handler: (transaction: QueryExecutor) => void) {}
function typed(transaction: QueryExecutor) { transaction('SELECT 1') }
function untyped(transaction: unknown) {}
declare const opaque: unknown
receive(read, typed) // CLEAR:read
receive(read, opaque) // ABSTAIN:read
receive(read, untyped) // ABSTAIN:read
receive(read, (transaction) => {}) // ABSTAIN:read
read('SELECT 2') // CLEAR_CALL
