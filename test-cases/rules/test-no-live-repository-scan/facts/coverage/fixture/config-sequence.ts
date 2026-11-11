export default { test: { projects: [
  { test: { name: "unknown object", sequence: dynamicSequence } },
  { test: { name: "spread object", sequence: { ...dynamicSequence } } },
  { test: { name: "computed key", sequence: { [key]: true } } },
  { test: { name: "unrelated key", sequence: { shuffle: true } } },
  { test: { name: "known false", sequence: { concurrent: false } } }
] } };
