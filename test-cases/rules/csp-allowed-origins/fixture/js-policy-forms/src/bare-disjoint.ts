const tags = ["alpha", "beta"];
const sep = "; ";
const directives = [
  "default-src 'self'",
  "script-src 'self' https://shared.example.com",
];
export const spread = [...tags];
export const spreadJoin = [...tags].join(";");
export const joined0 = ["script-src 'self' https://shared.example.com"].join();
export const joinedDyn = ["alpha"].join(sep);
export const joinedEmpty = ["alpha"].join(";");
export const joinedTwo = ["alpha"].join(";", "extra");
export const joinedOpt = tags?.join(";");
export const joinedId = tags.join(";");
export const minus = 2 - 1;
export const dynLeft = sep + "x";
export const dynRight = "x" + sep;
export const exported = [directives, tags];
