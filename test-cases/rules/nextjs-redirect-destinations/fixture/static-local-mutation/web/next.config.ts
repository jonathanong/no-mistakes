const pairs = [['x', 'missing-route']];
export default { redirects() { const pairs = [['x', 'about']]; const alias = pairs[0]; alias[1] = choose(); return [{ destination: '/' }, ...pairs.map(([src, dst]) => ({ destination: `/${dst}` }))]; } };
