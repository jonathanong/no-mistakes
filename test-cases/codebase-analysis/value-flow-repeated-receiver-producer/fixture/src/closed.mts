import { closedReceiver, rhs } from '@vendor/producer';
function run() {
  closedReceiver.get('key');
  closedReceiver.get = rhs.get;
}
run();
run();
