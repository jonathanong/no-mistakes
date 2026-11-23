export const JOB = { name: "before-unknown-member-write" };
const box = { job: JOB };
declare const key: string;
box[key].name = "after-unknown-member-write";
