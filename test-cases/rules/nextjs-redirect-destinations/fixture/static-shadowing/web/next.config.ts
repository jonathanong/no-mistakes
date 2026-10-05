const pairs = [['x', 'missing-route']];
export default { redirects() { const pairs = [['x', 'about']]; return pairs.map(([src, dst]) => ({ destination: `/${dst}` })); } };
