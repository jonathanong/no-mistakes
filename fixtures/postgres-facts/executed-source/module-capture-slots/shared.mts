export const statements = ['/* module_initial */ SELECT 1']
export function read() { return statements[0] }
export function replaceWithAnnotated() {
  statements[0] = '/* module_after */ SELECT 2'
}
export function replaceWithUnannotated() {
  statements[0] = 'SELECT module_without_annotation'
}
