const tags = ["alpha", "beta"];
const sep = "; ";
const directives = [
  "default-src 'self'",
  "script-src 'self' https://shared.example.com",
];
export const spread = [...tags];
export const joined0 = ["script-src 'self' https://shared.example.com"].join();
export const joinedDyn = ["alpha"].join(sep);
export const joinedEmpty = ["alpha"].join(";");
export const exported = [directives, tags];
