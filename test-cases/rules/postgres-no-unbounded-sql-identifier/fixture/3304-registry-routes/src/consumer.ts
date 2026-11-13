import * as direct from "./catalog";
import * as nested from "./barrel";
import * as aliases from "./alias";
import { group } from "./barrel";
import { TABLES as notNamespace } from "./catalog";
import * as absent from "./not-present";
void [direct, nested, aliases, group, notNamespace, absent];
