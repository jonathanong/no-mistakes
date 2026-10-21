import { nullableEntity as wrap } from "./helpers";

type User = { id: string };

declare const serverApi: {
  get(path: string): Promise<User | null>;
};

export function localWrapper(): User | null {
  return wrap(serverApi.get("/users/1"));
}
