import { Worker, testOnly, default as DefaultWorker } from '../src/worker.mts';
import { Worker as OtherWorker } from '../src/other.mts';
void [Worker, testOnly, DefaultWorker, OtherWorker];
import ReadyDefault, { Ready, stillTestOnly } from '../src/declared.mts';
void [ReadyDefault, Ready, stillTestOnly];
