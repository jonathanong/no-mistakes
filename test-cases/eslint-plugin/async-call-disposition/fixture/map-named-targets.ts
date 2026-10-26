import * as jobs from "@app/jobs";

// Configured async targets return promises, even when their names match array methods.
await Promise.all([jobs.map(1)]);
await Promise.allSettled([jobs.flatMap(2)]);
