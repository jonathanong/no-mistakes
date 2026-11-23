import { STRING, QUEUE_NAME } from "./mutable";
import { JOB as STATIC } from "./member-static";
import { JOB as COMPUTED } from "./member-computed";
import { JOB as UNKNOWN } from "./member-unknown";
import { JOB_ALIAS, STRING_ALIAS } from "./alias-mutated";
import { JOB as LET_SHADOW } from "./shadow-let";
import { JOB as COMPUTED_SHADOW } from "./shadow-computed";
import { JOB as UNSUPPORTED_SHADOW } from "./shadow-unsupported";
import { JOB as FORWARDED_SHADOW } from "./shadow-forwarded";
import { STRING as STRING_SHADOW } from "./shadow-string";
import { makeEnqueue, makeStringEnqueue, makeNamedEnqueue } from "./wrappers";

const primitiveProperty = makeEnqueue({ name: STRING });
primitiveProperty({});
const primitive = makeStringEnqueue(STRING);
primitive({});
const named = makeNamedEnqueue(QUEUE_NAME, "mutable-queue-job");
named({});
const staticMember = makeEnqueue(STATIC);
staticMember({});
const computedMember = makeEnqueue(COMPUTED);
computedMember({});
const unknownMember = makeEnqueue(UNKNOWN);
unknownMember({});
const importedAlias = makeEnqueue(JOB_ALIAS);
importedAlias({});
const importedStringAlias = makeStringEnqueue(STRING_ALIAS);
importedStringAlias({});
const letShadow = makeEnqueue(LET_SHADOW);
letShadow({});
const computedShadow = makeEnqueue(COMPUTED_SHADOW);
computedShadow({});
const unsupportedShadow = makeEnqueue(UNSUPPORTED_SHADOW);
unsupportedShadow({});
const forwardedShadow = makeEnqueue(FORWARDED_SHADOW);
forwardedShadow({});
const stringShadow = makeStringEnqueue(STRING_SHADOW);
stringShadow({});


import { STRING as REASSIGNED_CONST } from "./const-reassigned";
const reassignedConst = makeStringEnqueue(REASSIGNED_CONST);
reassignedConst({});


import { JOB_ALIAS as SHADOWED_JOB, STRING_ALIAS as SHADOWED_STRING } from "./alias-shadowed";
const shadowedImport = makeEnqueue(SHADOWED_JOB);
shadowedImport({});
const shadowedStringImport = makeStringEnqueue(SHADOWED_STRING);
shadowedStringImport({});
