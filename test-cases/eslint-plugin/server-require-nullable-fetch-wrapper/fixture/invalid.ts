type User = { id: string };
type MaybeUser = User | null;

declare const serverApi: {
  get(path: string): Promise<User | null>;
};

export function direct(): User | null {
  return serverApi.get("/users/1");
}

export const arrow = (): Promise<User | null> => serverApi.get("/users/2");

export const typed: () => MaybeUser = () => {
  return serverApi.get("/users/3") as MaybeUser;
};

export default function defaultDirect(): Promise<User | null> {
  return serverApi.get("/users/4");
}

export function optionalGetter(): User | null {
  return serverApi?.get("/users/5");
}
