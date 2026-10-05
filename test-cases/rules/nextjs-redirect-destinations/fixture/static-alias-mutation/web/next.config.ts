const pairs = [['x', 'about']];
const alias = pairs;
alias.push(getPair());
export default { redirects() { return [{ destination: '/' }, ...pairs.map(([src, dst]) => ({ destination: `/${dst}` }))]; } };
