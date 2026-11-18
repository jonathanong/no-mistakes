import sql from 'sql-template-strings';
export const customFields = sql`w.id`;
export const escapedFields = sql`w.id`;
export const overriddenFields = sql`w.id`;
export const mixedFields = sql`w.id`;
export const openFields = sql`${process.env.TABLE}`;
export function copyFields(prefix: ReturnType<typeof sql>) {
  return prefix.append(mixedFields);
}
export const calledFields = sql`w.id`;
export function copyCalled(prefix: ReturnType<typeof sql>) {
  return prefix.append(calledFields);
}
export const appliedFields = sql`w.id`;
export function copyApplied(prefix: ReturnType<typeof sql>) {
  return prefix.append(appliedFields);
}
