import { connection as left, connection as right } from '@example/db'

// The two locals refer to one exported object; a write through either affects both.
left.query('SELECT id FROM export_before')
right.query = httpQuery
left.query('SELECT id FROM export_after')
