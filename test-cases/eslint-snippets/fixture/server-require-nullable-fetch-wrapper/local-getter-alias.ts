import { nullableEntity as wrap } from "./helpers";

type User = { id: string };

declare function getUser(): Promise<User | null>;

export function wrapped(): User | null {
  return wrap(getUser());
}

export function bare(): User | null {
  return getUser();
}
