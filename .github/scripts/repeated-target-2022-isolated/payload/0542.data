import { read } from '@example/db'
import { runOrdered } from './ordered-helper.ts'

function statement(ctes: string) {
  return `/* ordered/list */ WITH ${ctes} SELECT id FROM input_data`
}

function readRows(statement: string) {
  return read(statement)
}

export function listOrdered() {
  return runOrdered({ statement, readRows })
}
