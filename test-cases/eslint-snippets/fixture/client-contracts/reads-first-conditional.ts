import { cache as kv } from '@app/cache';

async function conditionalFirst(flag: boolean, cached: unknown) {
  {
    flag && await kv.get('conditional');
    await kv.get('after-and');
  }
  {
    cached || await kv.get('conditional');
    await kv.get('after-or');
  }
  {
    cached ?? await kv.get('conditional');
    await kv.get('after-nullish');
  }
  {
    flag ? await kv.get('yes') : 'no';
    await kv.get('after-consequent');
  }
  {
    flag ? 'yes' : await kv.get('no');
    await kv.get('after-alternate');
  }
}
