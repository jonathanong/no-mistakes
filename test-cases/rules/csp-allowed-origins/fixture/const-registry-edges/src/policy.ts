import { REGISTERED_ORIGINS } from "./origins";

export const layered = [
  "img-src 'self' https://cdn.example.com",
  "https://plain.example.com",
  "img-src 'self' https://images.example.com",
];

export const spread = [...REGISTERED_ORIGINS, "https://extra.example.com"].join(" ");
