import { blank } from "./queues";
import { Queue } from "bullmq";

const permitted = new Queue("permitted", { defaultJobOptions: { removeOnFail: false } });
permitted.add("allowed-literal", {}, { removeOnComplete: false });

blank.add("allowed-missing", {}, { removeOnComplete: false });
