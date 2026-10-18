import { primary } from '@vendor/forward-scale-16';
function replacement() {}
let box = { client: primary };
function write0() { alias0[key].client.get = replacement; }
function write1() { alias1[key].client.get = replacement; }
function write2() { alias2[key].client.get = replacement; }
function write3() { alias3[key].client.get = replacement; }
function write4() { alias4[key].client.get = replacement; }
function write5() { alias5[key].client.get = replacement; }
function write6() { alias6[key].client.get = replacement; }
function write7() { alias7[key].client.get = replacement; }
function write8() { alias8[key].client.get = replacement; }
function write9() { alias9[key].client.get = replacement; }
function write10() { alias10[key].client.get = replacement; }
function write11() { alias11[key].client.get = replacement; }
function write12() { alias12[key].client.get = replacement; }
function write13() { alias13[key].client.get = replacement; }
function write14() { alias14[key].client.get = replacement; }
function write15() { alias15[key].client.get = replacement; }
const shared = box;
// Both branches reach one receiver; each physical writer projects it once.
const alias0 = { left: shared, right: shared };
const alias1 = { left: shared, right: shared };
const alias2 = { left: shared, right: shared };
const alias3 = { left: shared, right: shared };
const alias4 = { left: shared, right: shared };
const alias5 = { left: shared, right: shared };
const alias6 = { left: shared, right: shared };
const alias7 = { left: shared, right: shared };
const alias8 = { left: shared, right: shared };
const alias9 = { left: shared, right: shared };
const alias10 = { left: shared, right: shared };
const alias11 = { left: shared, right: shared };
const alias12 = { left: shared, right: shared };
const alias13 = { left: shared, right: shared };
const alias14 = { left: shared, right: shared };
const alias15 = { left: shared, right: shared };
write0();
write1();
write2();
write3();
write4();
write5();
write6();
write7();
write8();
write9();
write10();
write11();
write12();
write13();
write14();
write15();
function consume(value) { value.get('changed'); }
consume(primary);
