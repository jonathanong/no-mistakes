import { Queue } from "bullmq";
import { makeEnqueue } from "./factory";

const bare = new Queue("bare");
bare.add("known-missing", {});

const enqueue = makeEnqueue({ name: "welcome" });
enqueue({});
enqueue({}, { removeOnFail: false });
