import {
  createArrowResolved,
  createArrowUnresolved,
  createControlFlowFactory,
  createLocalUncertainFactory,
  createMixedFactory,
  createResolvedFactory,
  createUnrelatedFactory,
} from "./factory-unresolved";
import { createMixedFactory as createBarreledMixed } from "./factory-unresolved-barrel";

let mixed = createMixedFactory();
mixed.describe.serial("mixed factory", () => {});
mixed.describe.configure({ retries: 2 });

let controlFlow = createControlFlowFactory();
controlFlow.describe.serial("control-flow factory", () => {});

let localUncertain = createLocalUncertainFactory();
localUncertain.describe.serial("local uncertain factory", () => {});

let barreled = createBarreledMixed();
barreled.describe.serial("barreled mixed factory", () => {});

let arrowUnresolved = createArrowUnresolved();
arrowUnresolved.describe.serial("arrow unresolved factory", () => {});

const resolved = createResolvedFactory();
resolved.describe.serial("resolved factory", () => {});
resolved.describe.configure({ retries: 2 });

const arrowResolved = createArrowResolved();
arrowResolved.describe.serial("arrow resolved factory", () => {});

let unrelated = createUnrelatedFactory();
unrelated.describe.serial("unrelated factory", () => {});
unrelated.describe.configure({ retries: 2 });
