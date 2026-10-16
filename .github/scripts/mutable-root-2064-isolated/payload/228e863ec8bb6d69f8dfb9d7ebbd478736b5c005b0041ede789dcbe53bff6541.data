import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const sink = {};
// Eager static initializers run; cold instance initializers must not reset the root.
[sink[(box.client.get = replacement, 'key')]] = [class {
  static property = (box = { client: replica });
  static accessor eager = (box = { client: replica });
  cold: unknown = (box = { client: primary });
  accessor deferred: unknown = (box = { client: primary });
}];
function consume(value, changed) { value.get('primary'); changed.get('replica'); }
consume(primary, replica);
