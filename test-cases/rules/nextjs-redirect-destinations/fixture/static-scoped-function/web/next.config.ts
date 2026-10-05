const destination = '/missing-route';
function configure(destination: string, ...rest: unknown[]) {
 unknown(42); // An unrelated effect must not erase immutable destination values.
 const settings = { destination: '/' };
 { const destination = settings.destination;
   return { redirects() { return [{ destination }]; } };
 }
}
export default configure();
