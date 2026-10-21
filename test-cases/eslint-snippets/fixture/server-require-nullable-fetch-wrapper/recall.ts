import { serverApi, serverApi as api, "serverApi" as quoted } from "./server-api";

type User = { id: string };

export function renamed(): User | null {
  return api.get("/users/1");
}

export function bracket(): User | null {
  return api[`get`]("/users/2");
}

export function forwarded(): User | null {
  return api.get.call(api, "/users/3");
}

export function optional(): User | null {
  return api?.get("/users/4");
}

export function direct(): User | null {
  return serverApi.get("/users/5");
}

export function quotedGet(): User | null {
  return quoted.get("/users/6");
}

export function opaque(): User | null {
  return api.get.call(...["/users/7"]);
}
