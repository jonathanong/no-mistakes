const pairs = [['/old', '/missing-route']];
const result = mutate(pairs);
export default { redirects() { return [{destination: '/'}, ...pairs.map(([source,destination]) => ({source,destination}))]; } };
