import { unknown } from './target';
unknown();
// A recovered import must not make the skipped call-site extraction look complete.
return unknown;
