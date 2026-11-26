export async function loadProfile() {
  await fetch("/api/health");
  await fetch("https://app.example.com/api/session");
  return fetch("https://api.partner.example/me");
}
