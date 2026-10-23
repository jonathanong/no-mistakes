const enabled = true

export default {
  [`cacheComponents`]: enabled,
  cacheLife: {
    stale: 60,
  },
  cacheHandlers: {
    default: './cache-handler.js',
  },
}
