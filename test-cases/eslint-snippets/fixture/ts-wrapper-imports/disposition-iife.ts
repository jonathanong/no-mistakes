import { run } from "@example/jobs";

type Call = () => Promise<void>;

((() => run()) as Call)();
