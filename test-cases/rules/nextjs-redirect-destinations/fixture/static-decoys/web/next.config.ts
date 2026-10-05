const decoy = [{ destination: '/outside' }];
export default { decoy, redirects() { const other = [{ destination: '/also-outside' }]; const make = () => ({ destination: '/nested-decoy' }); return [{ destination: '/' }]; } };
