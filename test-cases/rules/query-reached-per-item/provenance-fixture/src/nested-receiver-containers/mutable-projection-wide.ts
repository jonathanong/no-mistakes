import { primary as client } from '@vendor/valkey';
let root = { client };
const wide = {
  branch00: root,
  branch01: root,
  branch02: root,
  branch03: root,
  branch04: root,
  branch05: root,
  branch06: root,
  branch07: root,
  branch08: root,
  branch09: root,
  branch10: root,
  branch11: root,
  branch12: root,
  branch13: root,
  branch14: root,
  branch15: root,
  branch16: root,
  branch17: root,
  branch18: root,
  branch19: root,
  branch20: root,
  branch21: root,
  branch22: root,
  branch23: root,
  branch24: root,
  branch25: root,
  branch26: root,
  branch27: root,
  branch28: root,
  branch29: root,
  branch30: root,
  branch31: root,
};
function key() { return 'unknown'; }
function replacement() {}
// One dynamic projection appends many successors before the root exits.
wide[key()].client.get = replacement;
