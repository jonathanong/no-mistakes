const routes = [{ destination: '/' }];
[0].forEach(() => routes.push({ destination: '/missing-route' }));
export default { redirects() { return [{ destination: '/' }, ...routes]; } };
