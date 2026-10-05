function config() {
 const routes = [{ destination: '/' }];
 if (unknown()) { const changed = mutate(routes); }
 return { redirects() { return [{ destination: '/' }, ...routes]; } };
}
export default config;
