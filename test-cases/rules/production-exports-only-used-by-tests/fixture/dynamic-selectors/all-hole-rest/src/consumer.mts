const [, ...rest] = await Promise.all([import("./worker.mjs"), import("./other.mjs")]);
