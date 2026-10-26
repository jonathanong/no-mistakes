// The root mapping must not answer the package config query.
export default { async redirects() { return [{ source: "/old", destination: "/wrong-root", permanent: true }]; } };
