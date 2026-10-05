const routes = [{ destination: '/' }];
class Setup { constructor() { routes.push({ destination: '/missing-route' }); } }
const instance = new Setup();
const tagged = setup`configure`;
setup`configure`;
export default { redirects() { return [{ destination: '/' }, ...routes]; } };
