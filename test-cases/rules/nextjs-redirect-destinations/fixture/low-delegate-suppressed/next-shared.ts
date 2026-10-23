export default {
  async redirects() {
    return [
      // no-mistakes-disable-next-line nextjs-redirect-destinations
      { source: '/old', destination: '/gone', permanent: true },
      { source: '/old', destination: '/missing', permanent: true }, // no-mistakes-disable-line nextjs-redirect-destinations
    ];
  },
};
