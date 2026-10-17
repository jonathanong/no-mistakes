import { primary, replica, other } from '@vendor/valkey';
function replacement() {}
let trigger = { client: other };
const sink = {};
declare const flag: boolean;
// A conditional var initializer joins the earlier root instead of replacing it.
[sink[(trigger.client.get = replacement, 'key')]] = [class {
  static {
    var holder = { client: primary };
    if (flag) { var holder = { client: replica }; }
    holder.client.get = replacement;
  }
}];
function consume(value, changed) { value.get('primary'); changed.get('replica'); }
consume(primary, replica);
