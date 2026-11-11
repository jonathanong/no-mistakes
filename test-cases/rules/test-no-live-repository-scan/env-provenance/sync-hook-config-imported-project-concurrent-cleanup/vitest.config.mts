import { parallel } from "./project.mjs";
export default { test: { projects: [parallel, { extends: true, test: { name: "serial", include: ["serial.mts"], sequence: { concurrent: false } } }] } };
