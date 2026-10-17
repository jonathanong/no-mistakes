import { primary, replica, other } from '@vendor/valkey';
function replacement() {}
let trigger = { client: other };
const sink = {};
// Earlier initializer reads must retain their root before a later var redeclaration.
[sink[(trigger.client.get = replacement, 'key')]] = [class {
  static {
    var holder = { client: primary }, saved = { original: holder.client }, holder = { client: replica };
    saved.original.get = replacement;
  }
}];
function consume(value, changed) { value.get('primary'); changed.get('replica'); }
consume(primary, replica);
