import * as first from './known';
import * as second from './known';
export default { test: { projects: [{ test: { ...first, ...second } }] } };
