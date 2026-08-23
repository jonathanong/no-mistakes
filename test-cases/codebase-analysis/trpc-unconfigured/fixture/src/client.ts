import { trpc } from "./trpc";

export async function loadUser() {
  return trpc.user.get.query();
}
