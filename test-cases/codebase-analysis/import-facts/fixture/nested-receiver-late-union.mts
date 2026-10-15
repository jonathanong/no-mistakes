import { primary, secondary } from '@vendor/valkey';
function replacement() {}
const left = { client: primary };
const right = { client: secondary };
left.client.get = replacement;
right.client.set = replacement;
left.client = {};
right.client = {};
left.client = primary;
right.extra = {};
right.other = {};
left[key] = primary;
right[key] = secondary;
[left.get, laterLeft = take(left.get)] = source;
[right.set, laterRight = take(right.set)] = source;
primary.info = replacement;
secondary.info = replacement;
