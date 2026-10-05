const settings = { destination: '/' };
const alias = settings;
alias.destination = '/missing-route';
// A read after initialization effects must not retain the old primitive value.
const destination = settings.destination;
export default { redirects() { return [{ destination: '/' }, { destination }]; } };
