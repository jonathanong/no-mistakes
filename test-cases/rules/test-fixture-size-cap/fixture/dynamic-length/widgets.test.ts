const extra = { length: 1 };
const key = "length";
let stored = 0;
const hiddenByGetter = Array.from({ length: 5000, get length() { return 1; } });
const hiddenBySetter = Array.from({
  length: 5000,
  set length(value) {
    stored = value;
  },
});
const hiddenBySpread = Array.from({ length: 5000, ...extra });
const hiddenByKey = Array.from({ length: 5000, [key]: 1 });
const keptName = Array.from({ length: 5000, ["name"]: 1 });
const keptIndex = Array.from({ length: 5000, [0]: 1 });
const keptAfterSpread = Array.from({ ...extra, length: 5000 });
