import { connection as left, other } from '@example/db'
import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// Passing one export is not a member write, so that export stays exact.
// Passing the module object can still replace every export's `.query`.
left.query('SELECT id FROM before_export_transfer')
sendAway(ns.connection)
left.query('SELECT id FROM after_export_transfer')
other.query('SELECT id FROM other_export_before_namespace_transfer')
sendAway(ns)
other.query('SELECT id FROM other_export_after_namespace_transfer')
