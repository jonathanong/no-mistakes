import { enqueueEmail } from "@app/jobs";
import api from "@app/consumer";
import * as ns from "@app/consumer";
const required = require("@app/consumer");
const alias = api;
alias.awaitReportedEnqueueAdmission = unknown;
ns.awaitReportedEnqueueAdmission++;
delete required.awaitReportedEnqueueAdmission;
api.awaitReportedEnqueueAdmission(() => enqueueEmail(1));
ns.awaitReportedEnqueueAdmission(() => enqueueEmail(2));
required.awaitReportedEnqueueAdmission(() => enqueueEmail(3));
import destructured from "@app/consumer";
import arrayTarget from "@app/consumer";
import loopTarget from "@app/consumer";
({ consume: destructured.awaitReportedEnqueueAdmission } = other);
[arrayTarget.awaitReportedEnqueueAdmission] = other;
for (loopTarget.awaitReportedEnqueueAdmission of other) {}
destructured.awaitReportedEnqueueAdmission(() => enqueueEmail(4));
arrayTarget.awaitReportedEnqueueAdmission(() => enqueueEmail(5));
loopTarget.awaitReportedEnqueueAdmission(() => enqueueEmail(6));
import restTarget from "@app/consumer";
import defaultTarget from "@app/consumer";
import keyTarget from "@app/consumer";
({ ...restTarget.awaitReportedEnqueueAdmission } = other);
[defaultTarget.awaitReportedEnqueueAdmission = fallback] = other;
for (keyTarget.awaitReportedEnqueueAdmission in other) {}
restTarget.awaitReportedEnqueueAdmission(() => enqueueEmail(7));
defaultTarget.awaitReportedEnqueueAdmission(() => enqueueEmail(8));
keyTarget.awaitReportedEnqueueAdmission(() => enqueueEmail(9));
