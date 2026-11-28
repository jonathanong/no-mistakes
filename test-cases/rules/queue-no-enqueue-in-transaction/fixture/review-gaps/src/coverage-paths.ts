import { withTx } from './tx'
import { enqueue } from '@sample/jobs'
import * as selfNs from './self-ns'
import { missingExport } from './tx'

function getTextA(): string {
  return 'a'
}

function getTextB(): string {
  return 'b'
}

export function edgeCoverage(
  flag: boolean,
  fnFactory: () => () => void,
  obj: { tag: (s: TemplateStringsArray) => string },
  extra?: string[],
) {
  let rep: unknown = 'x'
  rep = 'y'
  const deepDerived = (((((rep as string) + '1') + '2') + '3') + '4') + '5'
  const boundEnqueue = enqueue.bind(null)
  return withTx(async (tx) => {
    const d0: number[] = []
    const d1 = d0
    const d2 = d1
    const d3 = d2
    const d4 = d3
    const d5 = d4
    const d6 = d5
    d6.push(1)
    ;(flag ? getTextA : getTextB)().trim()
    fnFactory()()
    obj.tag`hello`
    ;((selfNs as unknown as { a: string }).a + 'x').trim()
    Math.abs.toString()
    ;(() => {}).toString()
    ;(flag ? extra : ['a'])?.map(tx.nested.handle)
    ;(selfNs as unknown as { nested: () => void }).nested()
    ;(missingExport as () => void)()
    boundEnqueue({ id: '1' })
    return 'a'.replace('a', rep as string).replace('a', deepDerived)
  })
}
