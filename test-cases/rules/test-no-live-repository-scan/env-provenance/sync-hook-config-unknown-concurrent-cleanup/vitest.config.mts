const concurrent = Math.random() > 0.5;
export default { test: { include: ["root.mts"], sequence: { concurrent } } };
