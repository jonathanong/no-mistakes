const routes = [{ destination: '/' }];
function setup() { routes.push({ destination: '/missing-route' }); }
const unused = [0].map(function () { return '/'; });
const ignored = [0].map(() => { setup(); return { destination: '/' }; });
export default { redirects() { return [{ destination: '/' }, ...routes]; } };
