export const contentSecurityPolicy = [
  "default-src 'self'",
  "img-src 'self' https://cdn.example.com https://[2001:db8::1]",
  "connect-src 'self' http://[::1]:3000",
  "script-src 'self' https://scripts.example.com",
].join("; ");
