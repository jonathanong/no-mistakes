import Plugin from "./plugin";
import * as plugins from "./namespace";
import { registry } from "./registry";

// Default import used as a registrant; namespace import collected too.
registry.register(new Plugin());
registry.register(new Plugin());
