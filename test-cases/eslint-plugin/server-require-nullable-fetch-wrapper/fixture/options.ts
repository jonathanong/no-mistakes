type User = { id: string };
type EntityResult = User;

declare const client: {
  fetchEntity(path: string): Promise<User | null>;
};
declare function asNullable<T>(value: T): T | null;

export function hinted(): EntityResult {
  return client.fetchEntity("/users/1") as EntityResult;
}

export function inferred(): EntityResult {
  return client.fetchEntity("/users/2") as EntityResult;
}

export function wrapped(): EntityResult {
  return asNullable(client.fetchEntity("/users/3")) as EntityResult;
}
