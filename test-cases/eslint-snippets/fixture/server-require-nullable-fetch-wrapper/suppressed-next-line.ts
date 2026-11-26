import { serverApi } from "./server-api";

type User = { id: string };

export function load(): User | null {
  // no-mistakes-disable-next-line server-require-nullable-fetch-wrapper -- retained probe
  return serverApi.get("/users/1");
}
