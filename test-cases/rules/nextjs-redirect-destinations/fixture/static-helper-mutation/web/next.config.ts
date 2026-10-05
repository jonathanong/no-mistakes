const routes = [{ destination: '/' }];
function setup() { routes.push({ destination: '/missing-route' }); }
setup();
export default { redirects() { return [{ destination: '/' }, ...routes]; } };
