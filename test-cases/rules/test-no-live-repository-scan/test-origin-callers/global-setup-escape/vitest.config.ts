import setup from "./src/global.mts";
export const setupCallback = setup;
export default { test: { include: ["tests/**/*.test.mts"], globalSetup: ["./src/global.mts"] } };
