export const contentSecurityPolicy = [
  `default-src 'self'`,
  "script-src 'self' https://scripts.example.com",
  "style-src 'self' https://styles.example.com",
  "font-src 'self' https://fonts.example.com",
  "img-src 'self' https://cdn.example.com https://*.assets.example.org",
  "connect-src 'self' https://api.example.com wss://rt.example.com:8443",
  "frame-src https://embeds.example.com",
].join("; ");
