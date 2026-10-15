import { primary } from '@vendor/valkey';
function replacement() {}
const prev0 = { client: primary };
const { client: prev1 = prev0.client } = prev0;
const { client: prev2 = prev1.client } = prev1;
const { client: prev3 = prev2.client } = prev2;
const { client: prev4 = prev3.client } = prev3;
const { client: prev5 = prev4.client } = prev4;
const { client: prev6 = prev5.client } = prev5;
const { client: prev7 = prev6.client } = prev6;
const { client: prev8 = prev7.client } = prev7;
const { client: prev9 = prev8.client } = prev8;
const { client: prev10 = prev9.client } = prev9;
const { client: prev11 = prev10.client } = prev10;
const { client: prev12 = prev11.client } = prev11;
prev12.get = replacement;
