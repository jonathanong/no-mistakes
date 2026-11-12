const { readdirSync } = require('node:fs');
const cache = require.cache;
readdirSync(process.env.ROOT, 'require-cache');
