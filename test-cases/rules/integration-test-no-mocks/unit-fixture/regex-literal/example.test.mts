expect(generated).toMatch(/from "msw"/);
expect(generated).toMatch(/require\('nock'\)/);
const real = require('sinon');
