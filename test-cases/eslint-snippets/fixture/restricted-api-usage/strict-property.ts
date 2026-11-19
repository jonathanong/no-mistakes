const config = {
  blockingWait: true,
  "blockingWait2": 1,
  blockingWait3() {},
  [dynamicKey]: 1,
  allowed: 2,
};
class Holder {
  blockingWait = 1;
  blockingWait4() {}
  other = 2;
}
const { blockingWait: copied } = config;
