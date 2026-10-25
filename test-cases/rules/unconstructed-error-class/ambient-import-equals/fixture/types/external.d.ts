// This external import makes the file a module. Its declaration below is an
// augmentation, which cannot prove a standalone missing module's ancestry.
import Driver = require('fixture:dependency');
declare module 'fixture:augmentation' {
  export class Base {}
}
