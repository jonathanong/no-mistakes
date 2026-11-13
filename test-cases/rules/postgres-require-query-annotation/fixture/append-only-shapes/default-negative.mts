import sql from 'sql-template-strings'
import { read } from '@bench/db'
function getter(q, o = { get enabled() { return opaque(q) } }) { q.append(' WHERE true') }
function computed(q, o = { [key()]: true }) { q.append(' WHERE true') }
function spread(q, o = { ...defaults }) { q.append(' WHERE true') }
function call(q, o = load()) { q.append(' WHERE true') }
function first(q = sql`SELECT 2`) { q.append(' WHERE true') }
export function getterDefault() { const q = sql`/* getter */ SELECT 1`; getter(q); read(q) }
export function computedDefault() { const q = sql`/* computed */ SELECT 1`; computed(q); read(q) }
export function spreadDefault() { const q = sql`/* spread */ SELECT 1`; spread(q); read(q) }
export function callDefault() { const q = sql`/* call */ SELECT 1`; call(q); read(q) }
export function firstDefault() { const q = sql`/* first */ SELECT 1`; first(q); read(q) }
