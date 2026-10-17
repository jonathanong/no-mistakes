import { primary, replica } from '@vendor/valkey';
function replacement() {}
function write(receiver) {
  const box = { wrapper: {} };
  box.wrapper = receiver;
  box.wrapper.get = replacement;
  receiver.get("inside");
}
write(replica);
function consume(value, changed) { value.get('original'); changed.get('changed'); }
consume(primary, replica);
