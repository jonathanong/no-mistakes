function config() {
 const routes = [{ destination: '/' }];
 return transform({ redirects() { return [{ destination: '/' }, ...routes]; } });
}
export default config;
