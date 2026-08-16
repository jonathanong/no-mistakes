import app from '../app';
import { service1 } from '../services/service-1';
app.route('/api/v1/resource-1/:id').get(() => service1('read-1')).patch(() => service1('patch-1'));
app.post('/api/v1/resource-1', () => service1('create-1'));
