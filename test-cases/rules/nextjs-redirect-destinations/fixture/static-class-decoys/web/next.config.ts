class Decoy { redirects; other() { return '/'; } }
const decoy = { redirects: 42 };
class Config { redirects = () => [{ destination: '/' }]; later = () => []; }
export default new Config();
