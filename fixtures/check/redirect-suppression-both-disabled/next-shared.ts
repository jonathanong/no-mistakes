// no-mistakes-disable-file nextjs-redirect-destinations
export default { async redirects() { return [{ source: '/old', destination: '/missing', permanent: true }]; } };
