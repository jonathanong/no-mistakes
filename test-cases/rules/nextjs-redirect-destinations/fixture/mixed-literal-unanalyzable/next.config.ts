const id = "dyn";
const cond = true;

const nextConfig = {
  async redirects() {
    return [
      {
        source: "/old-lit",
        destination: "/missing-literal",
        permanent: true,
      },
      {
        source: "/old-tpl",
        destination: `/missing-tpl`,
        permanent: true,
      },
      // `id` and `cond` stay identifiers so low cannot decide these rows.
      {
        source: "/old-interp",
        destination: `/x/${id}`,
        permanent: true,
      },
      {
        source: "/old-cond",
        destination: cond ? "/about" : "/nope",
        permanent: true,
      },
    ];
  },
  async rewrites() {
    return [
      {
        source: "/rw-lit",
        destination: "/missing-rw-literal",
      },
      {
        source: "/rw-tpl",
        destination: `/missing-rw-tpl`,
      },
      {
        source: "/rw-interp",
        destination: `/y/${id}`,
      },
      {
        source: "/rw-cond",
        destination: cond ? "/about" : "/nope",
      },
    ];
  },
};

export default nextConfig;
