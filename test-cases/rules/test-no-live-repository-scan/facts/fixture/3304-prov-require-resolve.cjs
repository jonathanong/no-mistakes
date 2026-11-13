const { readdirSync } = require('node:fs');
const resolved = require.resolve('node:fs');
readdirSync(process.env.ROOT, 'require-resolve');
