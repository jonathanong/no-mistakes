module.exports = {
  async redirects() {
    return [
      {
        source: "/old",
        destination: "/about",
        permanent: true,
      },
    ];
  },
};
