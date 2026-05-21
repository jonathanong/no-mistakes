import app from '../app.mts';
import { service3 } from '../services/service-3.mts';
app.route('/api/v1/resource-3/:id').get(() => service3('read-3')).patch(() => service3('patch-3'));
app.post('/api/v1/resource-3', () => service3('create-3'));
