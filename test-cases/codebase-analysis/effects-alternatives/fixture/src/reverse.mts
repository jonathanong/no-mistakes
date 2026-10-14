import { replica as receiver, primary } from '@vendor/database-reverse';
// Repeated calls revisit one physical imported-receiver occurrence.
function invoke() {
  receiver.write();
  receiver.write = primary.read;
}
invoke();
invoke();
