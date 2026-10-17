export let statement = '/* module_initial */ SELECT 1'
export function read() { return statement }
export function replace() { statement = 'SELECT branch_write_without_annotation' }
