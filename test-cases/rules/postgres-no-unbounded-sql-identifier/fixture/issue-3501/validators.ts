export function assertIdent(value: string): string {
  if (!/^[a-z_]+$/.test(value)) throw new Error('invalid identifier');
  return value;
}
