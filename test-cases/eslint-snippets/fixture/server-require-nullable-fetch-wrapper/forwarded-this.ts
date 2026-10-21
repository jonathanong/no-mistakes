import { serverApi } from "./server-api";
import { nullableEntity } from "./helpers";

type User = { id: string };

export function thisArg(): User | null {
  return nullableEntity.call(serverApi.get("/users/1"));
}

export function emptyApply(): User | null {
  return nullableEntity.apply(serverApi.get("/users/2"), []);
}

export function forwarded(): User | null {
  return nullableEntity.call(null, serverApi.get("/users/3"));
}

export function applied(): User | null {
  return nullableEntity.apply(null, [serverApi.get("/users/4")]);
}
