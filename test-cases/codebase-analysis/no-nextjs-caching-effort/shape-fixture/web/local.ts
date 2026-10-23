'use strict';
// These exports and assignments belong to an ordinary module, not route/config surfaces.
export const fetch = () => {};
fetch('/local', { cache: 'force-cache' });
export { fetch };
export default fetch;
module.exports = { cacheComponents: true };
