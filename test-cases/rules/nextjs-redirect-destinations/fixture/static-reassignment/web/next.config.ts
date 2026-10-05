let pairs = [['x', 'about']];
pairs = getPairs();
export default { redirects() { return [{ destination: '/' }, ...pairs.map(([src, dst]) => ({ destination: `/${dst}` }))]; } };
