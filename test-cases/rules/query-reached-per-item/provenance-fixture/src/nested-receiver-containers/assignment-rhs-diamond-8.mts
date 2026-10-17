import { primary, replica } from '@vendor/valkey';
function replacement() {}
const common = { wrapper: {} };
const retained = { client: primary };
const level0 = { left: common, right: common };
const level1 = { left: level0, right: level0 };
const level2 = { left: level1, right: level1 };
const level3 = { left: level2, right: level2 };
const level4 = { left: level3, right: level3 };
const level5 = { left: level4, right: level4 };
const level6 = { left: level5, right: level5 };
const level7 = { left: level6, right: level6 };
level7[globalThis.key][globalThis.key][globalThis.key][globalThis.key][globalThis.key][globalThis.key][globalThis.key][globalThis.key].wrapper = retained;
common.wrapper.client.get = replacement;
