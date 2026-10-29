import { read } from '@example/db'
import { fail } from './thrower.mjs'

export function badSelector(mode: string) { switch (mode + 'x') { case 'a': return 'a'; default: throw 'bad' } }
export function defaultAfterLabel(mode: string) { switch (mode) { case 'a': return 'a'; default: throw 'bad' } }
export function caseAfterDefault(mode: string) { switch (mode) { default: throw 'bad'; case 'a': return 'a' } }
export function nonStringCase(mode: string) { switch (mode) { case 1: return 'a'; default: throw 'bad' } }
export function duplicateCase(mode: string) { switch (mode) { case 'a': return 'a'; case 'a': return 'b'; default: throw 'bad' } }
export function nonReturnArm(mode: string) { switch (mode) { case 'a': throw 'a'; default: throw 'bad' } }
export function fallThrough(mode: string) { switch (mode) { case 'a': default: throw 'bad' } }
export function noReturn(mode: string) { switch (mode) { case 'a': } }
export function emptySwitch(mode: string) { switch (mode) { default: throw 'bad' } }
export function nonCallDefault(mode: string) { switch (mode) { default: return 'bad'; case 'a': return 'a' } }
export function breakDefault(mode: string) { switch (mode) { default: break; case 'a': return 'a' } }
export function optionalCallDefault(mode: string) { switch (mode) { default: return fail?.(mode); case 'a': return 'a' } }
export function memberCallDefault(mode: string) { switch (mode) { default: return helper.fail(mode); case 'a': return 'a' } }
export function impureArgDefault(mode: string) { switch (mode) { default: return fail(query('bad')); case 'a': return 'a' } }
export function unknownCallDefault(mode: string) { switch (mode) { default: return unknown(mode); case 'a': return 'a' } }
export function safeThrow(mode: string) { switch (mode) { case 'a': return 'a'; default: throw 'bad' } }
export function safeCall(mode: string) { switch (mode) { case 'a': return 'a'; default: return fail(mode) } }

export function calls(mode: string) {
  read(badSelector(mode)); read(defaultAfterLabel(mode));
  read(caseAfterDefault(mode)); read(nonStringCase(mode)); read(duplicateCase(mode));
  read(nonReturnArm(mode)); read(fallThrough(mode)); read(noReturn(mode)); read(emptySwitch(mode));
  read(nonCallDefault(mode)); read(breakDefault(mode)); read(optionalCallDefault(mode)); read(memberCallDefault(mode));
  read(impureArgDefault(mode)); read(unknownCallDefault(mode)); read(safeThrow(mode)); read(safeCall(mode));
}
