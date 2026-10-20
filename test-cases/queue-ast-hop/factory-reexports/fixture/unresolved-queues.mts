import { createQueue } from "./lib.mts";

export const q = createQueue("nsq");
const dq = createQueue("dq");
export default dq;
export const q6 = createQueue("q6");
export const q3 = createQueue("q3");
