const routes = [{ destination: '/' }];
function setup() { routes.push({ destination: '/missing-route' }); return; }
const callback = () => routes.push({ destination: '/missing-route' });
const other = function () { routes.push({ destination: '/missing-route' }); return; };
export default { redirects() { return routes; } };
