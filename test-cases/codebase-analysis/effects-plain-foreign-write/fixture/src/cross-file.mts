import { primary as receiver } from '@vendor/plain-alternatives';
import './writer.mts';
// The only receiver write is in another file; planning must inspect it conservatively.
receiver.transaction();
