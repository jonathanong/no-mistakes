import { createWorker } from "../lib.mts";
import { QN } from "../names.mts";

export const w = createWorker(QN, async (job) => {
  switch (job.name) {
    case "vDirect":
    case "vAlias2":
    case "vChain":
    case "vFromStar":
    case "vAlias3":
    case "vNamespace":
    case "vImported":
    case "vLocal":
    case "vTemplate":
    case "unproduced":
      return;
  }
});
