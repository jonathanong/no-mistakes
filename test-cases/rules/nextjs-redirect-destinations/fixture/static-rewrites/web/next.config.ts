const pairs = [['x', 'missing-route']];
export default { redirects() { return []; }, rewrites() { return { beforeFiles: pairs.map(([src, dst]) => ({ destination: `/${dst}` })), afterFiles: [], fallback: [] }; } };
