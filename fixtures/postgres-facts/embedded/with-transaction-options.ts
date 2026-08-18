import { withTransactionOptions as txn } from '@data-stores/psql'

export const arrow = () => query(`SELECT ${1}`)

export function unused() {
  return txn
}
