export type OriginName = string;

export const REGISTERED_ORIGINS = [
  "connect-src 'self' https://api.example.com https://cdn.example.com https://static.example.com",
] as const;
