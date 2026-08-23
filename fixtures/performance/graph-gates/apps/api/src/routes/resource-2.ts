import app from '../app';
import { service2 } from '../services/service-2';
app.route('/api/v1/resource-2/:id').get(() => service2('read-2')).patch(() => service2('patch-2'));
app.post('/api/v1/resource-2', () => service2('create-2'));
