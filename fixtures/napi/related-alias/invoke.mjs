import cjs from "../../../packages/no-mistakes/index.js";
import { related } from "../../../packages/no-mistakes/index.mjs";

const options = { root: process.cwd(), files: ["c.mts"], jobs: 1 };
const cjsReport = await cjs.related(options);
const esmReport = await related(options);
const paths = [cjsReport, esmReport].map((report) => report.files.map((file) => file.path).sort());
process.stdout.write(JSON.stringify(paths));
