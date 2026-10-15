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
for (const iteration of [0, 1]) {
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
  fn('key16');
  fn('key17');
  fn('key18');
  fn('key19');
  fn('key20');
  fn('key21');
  fn('key22');
  fn('key23');
  fn('key24');
  fn('key25');
  fn('key26');
  fn('key27');
  fn('key28');
  fn('key29');
  fn('key30');
  fn('key31');
  fn('key32');
  fn('key33');
  fn('key34');
  fn('key35');
  fn('key36');
  fn('key37');
  fn('key38');
  fn('key39');
  fn('key40');
  fn('key41');
  fn('key42');
  fn('key43');
  fn('key44');
  fn('key45');
  fn('key46');
  fn('key47');
  fn('key48');
  fn('key49');
  fn('key50');
  fn('key51');
  fn('key52');
  fn('key53');
  fn('key54');
  fn('key55');
  fn('key56');
  fn('key57');
  fn('key58');
  fn('key59');
  fn('key60');
  fn('key61');
  fn('key62');
  fn('key63');
  fn('key64');
  fn('key65');
  fn('key66');
  fn('key67');
  fn('key68');
  fn('key69');
  fn('key70');
  fn('key71');
  fn('key72');
  fn('key73');
  fn('key74');
  fn('key75');
  fn('key76');
  fn('key77');
  fn('key78');
  fn('key79');
  fn('key80');
  fn('key81');
  fn('key82');
  fn('key83');
  fn('key84');
  fn('key85');
  fn('key86');
  fn('key87');
  fn('key88');
  fn('key89');
  fn('key90');
  fn('key91');
  fn('key92');
  fn('key93');
  fn('key94');
  fn('key95');
  fn('key96');
  fn('key97');
  fn('key98');
  fn('key99');
  fn('key100');
  fn('key101');
  fn('key102');
  fn('key103');
  fn('key104');
  fn('key105');
  fn('key106');
  fn('key107');
  fn('key108');
  fn('key109');
  fn('key110');
  fn('key111');
  fn('key112');
  fn('key113');
  fn('key114');
  fn('key115');
  fn('key116');
  fn('key117');
  fn('key118');
  fn('key119');
  fn('key120');
  fn('key121');
  fn('key122');
  fn('key123');
  fn('key124');
  fn('key125');
  fn('key126');
  fn('key127');
})(step0());
  sharedPhase.get = replacement;
}
