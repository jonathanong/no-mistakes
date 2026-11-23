import * as framework from "@jest/globals";

framework.expect(Date.now() - start).toBeLessThanOrEqual(200);
framework.jest.useFakeTimers();
