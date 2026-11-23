export const JOB = { name: "before-computed-member-write" };
const box = { job: JOB };
box["job"].name = "after-computed-member-write";
