import { MUTATED_DEF, ALIAS_MUTATED_DEF, STABLE_DEF } from "./definitions";
import { makeEnqueue } from "./wrappers";

const exportedMutation = makeEnqueue(MUTATED_DEF, {});
exportedMutation({});
const exportedAliasMutation = makeEnqueue(ALIAS_MUTATED_DEF, {});
exportedAliasMutation({});

const spec = { name: "beforeLocalMutation" };
spec.name = "afterLocalMutation";
const localMutation = makeEnqueue(spec, {});
localMutation({});

const localDef = { name: "beforeLocalAliasMutation" };
const defAlias = localDef;
defAlias.name = "afterLocalAliasMutation";
const aliasedMutation = makeEnqueue(localDef, {});
aliasedMutation({});

const importAlias = STABLE_DEF;
importAlias.name = "changedImport";
const importedMutation = makeEnqueue(STABLE_DEF, {});
importedMutation({});

const defaults = { attempts: 2, priority: 0, jobId: "bad:defaults" };
const defaultsAlias = defaults;
defaultsAlias.priority = 3;
const defaultMutation = makeEnqueue({ name: "mutated-defaults" }, defaults);
defaultMutation({}, { removeOnComplete: true });

const options = { attempts: 2, removeOnComplete: true, priority: 0, jobId: "bad:options" };
const optionsAlias = options;
delete optionsAlias.attempts;
const optionsMutation = makeEnqueue({ name: "mutated-options" }, {});
optionsMutation({}, options);

const counted = { attempts: 0, priority: 0, jobId: "bad:counted" };
counted.attempts++;
const updateMutation = makeEnqueue({ name: "mutated-update" }, counted);
updateMutation({}, { removeOnComplete: true });
