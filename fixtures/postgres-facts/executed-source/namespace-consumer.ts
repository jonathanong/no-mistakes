import { read } from '@example/db'
import * as statements from './priority-helper.js'

export function list() {
  return read(statements.query)
}
