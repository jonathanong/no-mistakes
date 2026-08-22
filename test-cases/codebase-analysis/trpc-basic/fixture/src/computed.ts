import { trpc } from "./trpc";

const ns = "user";
export async function computed() {
  return trpc[ns].get.query();
}

export async function unknown() {
  return trpc.user.missing.query();
}
