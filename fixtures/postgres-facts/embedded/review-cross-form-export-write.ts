import { connection as left, other } from '@example/db'
import * as ns from '@example/db'

declare const httpQuery: (value: unknown) => unknown

// Namespace property writes affect the same exported object as a named import.
left.query('SELECT id FROM before_export_write')
ns.connection.query = httpQuery
left.query('SELECT id FROM after_export_write')
other.query('SELECT id FROM other_export_unaffected')
ns['connection'].query = httpQuery
left.query('SELECT id FROM after_computed_export_write')
