import app from '../app.mts';
import { service0 } from '../services/service-0.mts';
app.route('/api/v1/resource-0/:id').get(() => service0('read-0')).patch(() => service0('patch-0'));
app.post('/api/v1/resource-0', () => service0('create-0'));
