let listedOrigins = ["https://api.example.com", "https://cdn.example.com"];

const pushedOrigins = ["https://api.example.com", "https://cdn.example.com"];
pushedOrigins.push("https://extra.example.net");

export function buildLetCsp() {
  return `connect-src 'self' ${listedOrigins.join(" ")}`;
}

export function buildPushedCsp() {
  return `connect-src 'self' ${pushedOrigins.join(" ")}`;
}
