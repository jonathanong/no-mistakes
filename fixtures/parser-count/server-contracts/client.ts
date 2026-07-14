export async function loadUsers() {
  return fetch("/api/users?include=posts");
}
