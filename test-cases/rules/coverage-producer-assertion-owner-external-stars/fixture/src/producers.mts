import { work as externalWork } from './external.mjs';
import { work as chainedWork } from './barrel.mjs';
import { work as shadowWork } from './shadow.mjs';
import { work as competingWork } from './competing.mjs';
import { work as competingLocalWork } from './competing-local.mjs';
import { work as cycleWork } from './cycle-a.mjs';
import { work as missingWork } from './missing.mjs';
import DefaultWork from './external.mjs';
import { ok } from './assertion.mjs';
import { readFileSync } from './fs.mjs';
import { readFileSync as directRead } from 'node:fs';
export function plainStar() { externalWork(); }
export function chainedStar() { chainedWork(); }
export function localExportShadow() { shadowWork(); }
export function localParameterShadow(externalWork: () => void) { externalWork(); }
export function competingStars() { competingWork(); }
export function competingLocalStar() { competingLocalWork(); }
export function cycle() { cycleWork(); }
export function missingSource() { missingWork(); }
export function defaultImport() { DefaultWork(); }
export function dottedMember() { externalWork.run(); }
export function assertionStar() { ok(true); }
export function fsStar() { readFileSync('result', 'utf8'); }
export function fsDirect() { directRead('result', 'utf8'); }
