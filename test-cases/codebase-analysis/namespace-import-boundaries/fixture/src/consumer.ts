import { Known as Good } from './mixed';
import { Known as Absent } from './absent';
import { Known as Other } from './other';
import { Known as External } from './external';
import { Known as Cycle } from './cycle-a';
import { Known as Missing } from './missing';
import * as Module from './namespaces';
export function good() { return new Good.Item(); }
export function absent() { return new Absent.Item(); }
export function other() { return new Other.Item(); }
export function external() { return new External.Item(); }
export function cycle() { return new Cycle.Item(); }
export function missing() { return new Missing.Item(); }
export function moduleValue() { return new Module.Known(); }
