import app from '../app.mts';
import { service1 } from '../services/service-1.mts';
app.route('/api/v1/resource-1/:id').get(() => service1('read-1')).patch(() => service1('patch-1'));
app.post('/api/v1/resource-1', () => service1('create-1'));
