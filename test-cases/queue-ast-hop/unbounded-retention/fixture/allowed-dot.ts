import { bare } from "./queues";

// Exempted by queues.unboundedJobRetention.allowPaths ("./allowed-dot.ts").
bare.add("allowed-dot-script", {}, { removeOnComplete: true, removeOnFail: false });
