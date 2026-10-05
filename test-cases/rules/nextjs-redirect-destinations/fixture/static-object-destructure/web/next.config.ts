const destinations = [{ destination: '/' }];
export default { redirects() {
 const [{ destination }] = destinations;
 return [{ destination }];
} };
