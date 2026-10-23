export default {
  async redirects() {
    return [{ source: "/a", destination: "/gone", permanent: true }];
  },
};
