import { read } from '@example/db'
import {
  buildAfterCallGate,
  buildAfterObjectGate,
  buildAfterTaggedGate,
} from './async-suspended-builder.ts'

export async function readAfterObjectGate() {
  return read(await buildAfterObjectGate())
}

export async function readAfterTaggedGate() {
  return read(await buildAfterTaggedGate())
}

export async function readAfterCallGate() {
  return read(await buildAfterCallGate())
}
