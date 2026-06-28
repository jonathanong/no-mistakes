import { usersHref } from "./links";
import { useRouter } from "next/navigation";

export async function loadUsers() {
  return fetch("/api/users?include=posts&sort=name");
}

export async function createUser() {
  return fetch("/api/users?include=posts&sort=name", { method: "POST" });
}

export function navigateUsers() {
  const router = useRouter();
  router.push(usersHref());
}
