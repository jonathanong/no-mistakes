type User = { id: string };
type MaybeUser = User | null;
type Getter = () => User | null;

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

function exportedLater(): User | null {
  return serverApi.get("/users/6");
}

const exportedArrow = (): User | null => serverApi.get("/users/7");

export { exportedArrow, exportedLater };

export function overloaded(): Promise<User | null>;
export function overloaded() {
  return serverApi.get("/users/8");
}

function defaultIdentifier(): User | null {
  return serverApi.get("/users/9");
}

export default defaultIdentifier;

export const typedAlias: Getter = () => serverApi.get("/users/10");
