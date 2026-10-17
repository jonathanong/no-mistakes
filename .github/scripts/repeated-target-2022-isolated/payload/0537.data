import sql from 'sql-template-strings'
import { read } from '@example/db'
import { appendKnown, appendLater, makeQuery } from './review-safety-helper.js'

export function authoredMarker() {
  return read("/* authored marker */ SELECT 'sql_placeholder_1'")
}

export function generatedMarker(id: string) {
  return read(sql`/* generated marker */ SELECT ${id}`)
}

export function getterArgument() {
  const query = sql`/* getter argument */ SELECT 1`
  const object = {
    get trigger() {
      query.append(' UPDATE posts SET title = title')
      return 1
    },
  }
  appendKnown(query, object.trigger)
  return read(query)
}

export function shadowedHelper(appendKnown: (query: typeof sql) => void) {
  const query = makeQuery()
  appendKnown(query)
  return read(query)
}

export function invokeShadowed() {
  return shadowedHelper(query => query['unknownMethod']())
}

export function computedRecord(key: string) {
  const builders = {
    '': () => '/* empty key */ SELECT 1',
    other: () => '/* other key */ UPDATE posts SET title = title',
  }
  return read(builders[key]())
}

export function boundRead(id: string) {
  return read(sql`/* bound read */ WITH rows AS (SELECT ${id}::uuid AS id) SELECT id FROM rows`)
}

export function boundWrite(id: string) {
  return read(sql`/* bound write */ WITH rows AS (UPDATE posts SET title = ${id} RETURNING id) SELECT id FROM rows`)
}

export function taggedGetterArgument() {
  const query = sql`/* tagged getter */ SELECT 1`
  const object = {
    get trigger() {
      query.append(' UPDATE posts SET title = title')
      return 1
    },
  }
  appendKnown(query, sql`${object.trigger}`)
  return read(query)
}

export function customAppendReceiver() {
  const query = sql`/* custom receiver */ SELECT 1`
  const receiver = {
    append(_suffix: string) {
      query.append(' UPDATE posts SET title = title')
    },
  }
  appendKnown(receiver)
  return read(query)
}

export function directTaggedGetter() {
  const query = sql`/* direct getter */ SELECT 1`
  const object = {
    get trigger() {
      query.append(' UPDATE posts SET title = title')
      return 1
    },
  }
  query.append(sql`${object.trigger}`)
  return read(query)
}

export function twoBindTemplates(first: string, second: string) {
  const query = sql`/* two binds */ SELECT ${first}::text`
  query.append(sql` || ${second}::text`)
  return read(query)
}

export function unawaitedAppend() {
  const query = sql`/* async append */ SELECT 1`
  const pending = appendLater(query)
  void pending
  return read(query)
}
