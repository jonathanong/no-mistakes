const object = { routes: [{ destination: '/' }] };
const unrelated = { destination: '/' };
const alias = object.routes;
alias[0] = { destination: '/missing-route' };
export default { redirects() { return [{ destination: '/' }, ...object.routes]; } };
