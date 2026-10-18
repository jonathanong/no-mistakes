import { connection as left, other } from '@example/db'
import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// A property transfer affects that export; a namespace transfer may affect all exports.
left.query('SELECT id FROM before_export_transfer')
sendAway(ns.connection)
left.query('SELECT id FROM after_export_transfer')
other.query('SELECT id FROM other_export_before_namespace_transfer')
sendAway(ns)
other.query('SELECT id FROM other_export_after_namespace_transfer')
