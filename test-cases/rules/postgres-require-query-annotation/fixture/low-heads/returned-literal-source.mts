import sql from 'sql-template-strings'
import native from 'sql-template-strings'
import other from './custom.mjs'

export function acceptedStrings(select) {
  const statement = select.append(' columns').append(' FROM records'); return statement
}

export function acceptedTags(select) {
  const statement = select.append(sql` columns`).append(native` FROM records`); return statement
}

export function acceptedEight(select) {
  const statement = select.append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x'); return statement
}

export function nine(select) {
  const statement = select.append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x'); return statement
}

export function customTag(select) {
  const statement = select.append(other` columns`); return statement
}

export function shadowedTag(sql) {
  const statement = sql.append(sql` columns`); return statement
}

export function interpolated(select) {
  const statement = select.append(sql` columns ${value}`); return statement
}

export function dynamicSuffix(select) {
  const statement = select.append(value); return statement
}

export function changed(select) {
  const statement = select.append(' columns'); return statement
}

export function aliasEscape(select) {
  const statement = select.append(' columns'); leak(statement); return statement
}

export function parameterEscape(select) {
  const statement = select.append(' columns'); leak(select); return statement
}

export function nonconst(select) {
  let statement = select.append(' columns'); return statement
}

export function wrongReturn(select) {
  const statement = select.append(' columns'); return select
}

export function noAppend(select) {
  const statement = select; return statement
}

export function optionalAppend(select) {
  const statement = select?.append(' columns'); return statement
}

export function computedAppend(select) {
  const statement = select['append'](' columns'); return statement
}

export function spreadAppend(select) {
  const statement = select.append(...parts); return statement
}

export function optionalCall(select) {
  const statement = select.append?.(' columns'); return statement
}

export function extraArguments(select) {
  const statement = select.append(' columns', ' more'); return statement
}

export function plainTemplate(select) {
  const statement = select.append(` columns`); return statement
}

export function defaultParameter(select = sql`SELECT `) {
  const statement = select.append(' columns'); return statement
}

export function extraParameter(select, extra) {
  const statement = select.append(' columns'); return statement
}

export function restParameter(...select) {
  const statement = select.append(' columns'); return statement
}

export function destructuredParameter({select}) {
  const statement = select.append(' columns'); return statement
}

export async function asyncFunction(select) {
  const statement = select.append(' columns'); return statement
}

export function* generatorFunction(select) {
  const statement = select.append(' columns'); return statement
}

changed = opaque
