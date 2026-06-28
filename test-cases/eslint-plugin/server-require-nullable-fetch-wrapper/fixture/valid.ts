type User = { id: string };
type MaybeUser = User | null;

declare const serverApi: {
  get(path: string): Promise<User | null>;
  post(path: string): Promise<User>;
};
declare function nullableEntity<T>(value: T): T | null;

function internal(): User | null {
  return serverApi.get("/users/1");
}

export function wrapped(): User | null {
  return nullableEntity(serverApi.get("/users/1"));
}

export function nonNullable(): User {
  return serverApi.get("/users/1") as User;
}

export function otherMethod(): User | null {
  return serverApi.post("/users/1") as User;
}

export function hinted(): MaybeUser {
  return nullableEntity(serverApi.get("/users/1"));
}

export default function defaultWrapped(): Promise<User | null> {
  return nullableEntity(serverApi.get("/users/1"));
}

export function nullableElements(): Promise<Array<User | null>> {
  return serverApi.get("/users/2") as Promise<Array<User | null>>;
}
