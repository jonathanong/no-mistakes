import { primary, replica } from '@vendor/valkey';
function replacement() {}
const common = { wrapper: {} };
const retained = { client: primary };
const level0 = { left: common, right: common };
const level1 = { left: level0, right: level0 };
const level2 = { left: level1, right: level1 };
const level3 = { left: level2, right: level2 };
level3[globalThis.key][globalThis.key][globalThis.key][globalThis.key].wrapper = retained;
common.wrapper.client.get = replacement;
