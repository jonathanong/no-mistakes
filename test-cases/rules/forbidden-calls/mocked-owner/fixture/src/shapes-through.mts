import { analyzeProject as starred } from './star-ok.mts'
import { localHelper } from './star-miss.mts'
import { LocalClass } from './local-class.mts'
import { analyzeProject as forwarded } from './import-forward.mts'
import { again } from './cycle-a.mts'
import { analyzeProject as dotted } from 'no-mistakes'
import { shared } from './ambiguous.mts'
import { helper } from './two-imports.mts'
import { helper as value } from './value-export.mts'
import { localHelper as parsed } from './parse-gap.mts'
import { localHelper as recovered } from './broken.mts'

export function run() {
  starred()
  localHelper()
  new LocalClass()
  forwarded()
  again()
  dotted.member()
  shared()
  helper()
  value()
  parsed()
  recovered()
}
