export async function loadUsers() {
  return fetch("/api/users?include=posts&sort=name");
}

export async function createUser() {
  return fetch("/api/users?include=posts&sort=name", { method: "POST" });
}
