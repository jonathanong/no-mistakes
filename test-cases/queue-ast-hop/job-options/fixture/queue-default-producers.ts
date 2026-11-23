import { enqueueBare } from "./factory";
import { primary } from "./queue-default-primary";
import { secondary } from "./queue-default-secondary";
import * as throughStar from "./queue-default-star";
import { selected } from "./queue-default-barrel";

primary.add("primary-defaults", {});
secondary.add("secondary-defaults", {});
secondary.add("secondary-covered", {}, { removeOnComplete: true });
enqueueBare(throughStar.selected, "through-star");
const alias = selected;
alias.add("through-alias", {});
