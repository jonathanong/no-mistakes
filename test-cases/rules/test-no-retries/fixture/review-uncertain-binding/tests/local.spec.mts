import { customTest } from "../helpers/fixture.mjs";
import { forward } from "../helpers/wrappers.mjs";

declare const useFixture: boolean;
const other = { describe: { configure(_options: object) {} } };
const selected = useFixture ? customTest : other;
selected.describe.configure({ retries: 2, mode: "serial" });

const direct = customTest;
direct.describe.configure({ retries: 2, mode: "serial" });

const forwarded = forward(customTest);
forwarded.describe.configure({ retries: 2, mode: "serial" });
