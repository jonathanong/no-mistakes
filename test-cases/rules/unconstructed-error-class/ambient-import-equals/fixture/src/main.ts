import { Base as ScriptBase } from 'fixture:script';
import { Base as AugmentedBase } from 'fixture:augmentation';
export class ScriptWorker extends ScriptBase<{ value: string }> {}
export class AugmentationChild extends AugmentedBase {}
