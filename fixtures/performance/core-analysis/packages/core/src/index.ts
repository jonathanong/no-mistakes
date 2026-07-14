export interface CoreValue {
  id: string;
}

export function formatCore(value: CoreValue): string {
  return value.id.toUpperCase();
}
