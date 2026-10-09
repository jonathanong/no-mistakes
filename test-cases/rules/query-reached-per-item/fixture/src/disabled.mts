// no-mistakes-disable-file query-reached-per-item -- bounded ordered migration
import { read } from 'configured-db';
export function migrate(items: unknown[]) { for (const item of items) read(item); }
