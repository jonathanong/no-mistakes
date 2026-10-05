const routes = [{ destination: '/' }];
function setup() { routes.push({ destination: '/missing-route' }); }
const ignored = { value: setup() };
export default { redirects() { return [{ destination: '/' }, ...routes]; } };
