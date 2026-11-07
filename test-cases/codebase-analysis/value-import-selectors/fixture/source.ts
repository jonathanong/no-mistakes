declare function createStatement(): unknown
export const TABLES = ['alpha', 'beta'] as const
export const statement = createStatement()
export const options = { table: 'alpha' } as const
export let mutable = createStatement()
export function run() { return 'ready' }
export type Only = 'alpha'
export default options
