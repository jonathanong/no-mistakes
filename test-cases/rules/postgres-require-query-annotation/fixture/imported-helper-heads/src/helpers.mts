import sql from 'sql-template-strings';

export function named(value: string) {
  return sql`/* importedNamed */ SELECT ${value}`;
}
export { named as renamed };

export const arrow = (value: string) => {
  return `/* importedArrow */ SELECT ${value}`;
};

export function missing() {
  return sql`SELECT 1`;
}

export function changed() {
  return sql`/* changed */ SELECT 1`;
}
changed = () => sql`SELECT 1`;

export function partial(value: string) {
  return sql`/* ${value} */ SELECT 1`;
}

export function early(flag: boolean) {
  if (flag) return sql`/* early */ SELECT 1`;
  return sql`/* early */ SELECT 2`;
}
