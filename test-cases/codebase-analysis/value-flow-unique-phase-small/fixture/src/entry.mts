import { sharedPhase } from '@vendor/valkey';
function replacement() {}
function acquire(fn = sharedPhase.get) { return fn; }
function step0() { return step1(); }
function step1() { return step2(); }
function step2() { return step3(); }
function step3() { return step4(); }
function step4() { return step5(); }
function step5() { return step6(); }
function step6() { return step7(); }
function step7() { return step8(); }
function step8() { return step9(); }
function step9() { return step10(); }
function step10() { return step11(); }
function step11() { return step12(); }
function step12() { return step13(); }
function step13() { return step14(); }
function step14() { return step15(); }
function step15() { return step16(); }
function step16() { return step17(); }
function step17() { return step18(); }
function step18() { return step19(); }
function step19() { return step20(); }
function step20() { return step21(); }
function step21() { return step22(); }
function step22() { return step23(); }
function step23() { return step24(); }
function step24() { return step25(); }
function step25() { return step26(); }
function step26() { return step27(); }
function step27() { return step28(); }
function step28() { return step29(); }
function step29() { return step30(); }
function step30() { return step31(); }
function step31() { return acquire(); }
(function consume(fn) {
  fn('key0');
  fn('key1');
  fn('key2');
  fn('key3');
  fn('key4');
  fn('key5');
  fn('key6');
  fn('key7');
  fn('key8');
  fn('key9');
  fn('key10');
  fn('key11');
  fn('key12');
  fn('key13');
  fn('key14');
  fn('key15');
})(step0(), sharedPhase.get = replacement);
