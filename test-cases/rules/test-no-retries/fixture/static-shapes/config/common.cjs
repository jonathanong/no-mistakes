const settings = { retries: 3, workers: 1 };
module.exports = settings;
exports = settings;
Object.keys(settings);
const cache = { previous: {} };
cache.previous.exports = { retries: 99, workers: 1 };
