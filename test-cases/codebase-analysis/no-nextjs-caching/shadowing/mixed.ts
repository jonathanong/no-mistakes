import { Of, In, For, Discriminant, Case, Body, Caught, Base, Static, Unused } from './components';
import './setup';
const mode = 'force-cache';
function inspect() {
  for (const value of (Of, import('./of'), items)) { Body; } // dynamic ./of
  for (let key in (In, import('./in'), items)) { Body; } // dynamic ./in
  for (let index = (For, import('./init'), 0); condition; index++) { Body; } // dynamic ./init
  // The discriminant must be visited exactly once by every enabled fact family.
  switch ((Discriminant, import('./discriminant'), fetch('/data', { cache: mode }))) { // dynamic ./discriminant
    case (Case, import('./case'), value): // dynamic ./case
      Body;
      import('./case-body'); // dynamic ./case-body
      break;
    default:
      let mode = 'no-store';
  }
  try {} catch (mode) {
    Caught;
    import('./catch'); // dynamic ./catch
  }
  class Local extends Base { static { Static; import('./static'); } } // dynamic ./static
}
const arrow = () => import('./arrow'); // dynamic ./arrow
const named = function () { return import('./function'); }; // dynamic ./function
