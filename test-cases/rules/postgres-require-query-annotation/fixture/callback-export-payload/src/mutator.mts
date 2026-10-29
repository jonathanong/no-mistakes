import defaulted, { direct, publicAlias } from './query.mts'
export function mutate() {
  direct.text = 'SELECT 2'
  publicAlias.text = 'SELECT 2'
  defaulted.text = 'SELECT 2'
}
