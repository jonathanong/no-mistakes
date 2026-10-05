const pairs = [['/old', '/missing-route']];
if (unknown()) pairs.push(['/other', '/']);
export default { redirects() { return [{destination: '/'}, ...pairs.map(([source,destination]) => ({source,destination}))]; } };
