import { read } from '@example/db'
import {
  baseline,
  spreadGate,
  indexGate,
  templateGate,
  childrenGate,
  containerGate,
  opaqueGate,
  writeGate,
  nestedWriteGate,
  appendGate,
} from './suspension-shapes'

export async function readBaseline() {
  return read(await baseline())
}

export async function readSpread() {
  return read(await spreadGate())
}

export async function readIndex() {
  return read(await indexGate())
}

export async function readTemplate() {
  return read(await templateGate())
}

export async function readChildren() {
  return read(await childrenGate())
}

export async function readContainer() {
  return read(await containerGate())
}

export async function readOpaque() {
  return read(await opaqueGate())
}

export async function readWrite() {
  return read(await writeGate())
}

export async function readNestedWrite() {
  return read(await nestedWriteGate())
}

export async function readAppend() {
  return read(await appendGate())
}
