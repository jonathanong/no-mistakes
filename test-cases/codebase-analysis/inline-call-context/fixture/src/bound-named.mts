import { primary } from '@vendor/valkey';
function helper({ client }) { client.get(); }
helper.bind(null)({ client: primary });
