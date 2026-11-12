import * as barrel from './control-barrel.mts';
barrel.STAGES[0].run = async () => ({ rowCount: 0 });
