// OXC retains the first base and class AST while diagnosing multiple extends bases.
import { Base } from 'external-base';
export class Broken extends Base, Other {}
