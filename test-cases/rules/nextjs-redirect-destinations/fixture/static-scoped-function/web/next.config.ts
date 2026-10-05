const destination = '/missing-route';
function configure(destination: string, ...rest: unknown[]) {
 const settings = { destination: '/' };
 { const destination = settings.destination;
   return { redirects() { return [{ destination }]; } };
 }
}
export default configure();
