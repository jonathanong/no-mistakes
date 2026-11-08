const [{ work }, { other }] = await Promise.all([import("./worker.mjs"), import("./other.mjs")]); work(); other();
