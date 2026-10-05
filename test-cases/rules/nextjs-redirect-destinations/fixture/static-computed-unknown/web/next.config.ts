const pairs = ['/'];
export default { redirects() { return [{ destination: '/' }, { destination: pairs['0'] }, { destination: pairs[0.5] }, { destination: pairs[99] }]; } };
