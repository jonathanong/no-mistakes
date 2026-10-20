import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
box = { client: replica };
[flag ? box : box].map((value, index, items) => { items[0].client.get = replacement; });
replica.get('changed');
primary.get('old');
unrelated.get('control');
