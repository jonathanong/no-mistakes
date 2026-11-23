import { bare } from "../queues";

// Exempted by queues.unboundedJobRetention.allowPaths.
bare.add("allowed-script", {}, { removeOnComplete: true, removeOnFail: false });
