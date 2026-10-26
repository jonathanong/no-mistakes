import { lookup as getUser } from "./api";

type User = { id: string };

export function bare(): User | null {
  return getUser("/users/1");
}
