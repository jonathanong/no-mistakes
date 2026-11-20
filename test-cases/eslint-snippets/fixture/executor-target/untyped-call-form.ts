import { type PoolClient } from "@example/db";

declare function readRow(client: unknown, sourceId: string): void;

export function withClient(client: PoolClient, sourceId: string) {
  readRow(client, sourceId);
}
