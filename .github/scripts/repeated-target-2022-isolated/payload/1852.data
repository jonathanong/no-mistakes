import { read } from '@example/db'
import { runOrdered } from './ordered-helper.ts'

const base = {
  statement: (ctes: string) => `/* ordered/spread */ WITH ${ctes} SELECT id FROM input_data`,
  readRows: (statement: string) => read(statement),
}

export function spreadCallbacks() {
  return runOrdered({ ...base })
}

export function getterCallbacks() {
  return runOrdered({
    get statement() {
      return (ctes: string) => `/* ordered/getter */ WITH ${ctes} SELECT id FROM input_data`
    },
    readRows: (statement: string) => read(statement),
  })
}

export function conditionalCallbacks(useWrite: boolean) {
  return runOrdered({
    statement: useWrite
      ? (ctes: string) => `/* ordered/write */ WITH ${ctes} DELETE FROM posts`
      : (ctes: string) => `/* ordered/read */ WITH ${ctes} SELECT id FROM input_data`,
    readRows: (statement: string) => read(statement),
  })
}
