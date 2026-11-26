import { serverApi } from "./server-api";

type User = { id: string };

export function load(): User | null {
  return serverApi.get("/users/1");
}
