import { randomUUID } from 'node:crypto';

export function createTempTableName(prefix: string): string {
  return `${prefix}_${randomUUID().replaceAll('-', '')}`;
}
