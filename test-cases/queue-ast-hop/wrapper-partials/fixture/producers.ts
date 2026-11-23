import { makePartial, makeOptions } from "./wrappers";

declare function getName(): string;

const pending = makePartial({ name: getName() }, { attempts: 2, priority: 3 });
pending({}, {});

const known = makePartial({ name: "known" }, { attempts: 2, priority: 3 });
known({}, {});

makeOptions("defaults", {}, { attempts: 2, priority: 3 });
makeOptions("zero", { options: { attempts: 2, priority: 0 } });
