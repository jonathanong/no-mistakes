export const JOB = { name: "before-static-member-write" };
const box = { job: JOB };
box.job.name = "after-static-member-write";
