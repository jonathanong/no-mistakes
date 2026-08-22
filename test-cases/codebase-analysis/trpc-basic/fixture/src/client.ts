import { trpc } from "./trpc";

export async function loadUser() {
  return trpc.user.get.query();
}

export async function createUser() {
  return trpc.user.create.mutate({});
}
