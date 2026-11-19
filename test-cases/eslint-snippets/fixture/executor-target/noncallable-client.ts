import { read, type PoolClient, type QueryWithClient, type SessionHandle } from "@example/db";

declare function addUrls(
  userId: string,
  inputs: readonly string[],
  options: { query: unknown; client: unknown },
): void;
declare function readRow(client: unknown, sourceId: string): void;
declare function publish(session: unknown, sourceId: string): void;

export function save(userId: string, inputs: readonly string[], query: QueryWithClient) {
  addUrls(userId, inputs, { query, client: query.client });
}

export function readX(transaction: { client: PoolClient }, sourceId: string) {
  readRow(transaction.client, sourceId);
}

export function withClient(client: PoolClient, sourceId: string) {
  readRow(client, sourceId);
}

export function notify(session: SessionHandle, sourceId: string) {
  publish(session, sourceId);
}

export function forwardRead(id: string, client: PoolClient) {
  addUrls(id, [id], { query: read, client });
}
