import app from '../app.mts';
import { service4 } from '../services/service-4.mts';
app.route('/api/v1/resource-4/:id').get(() => service4('read-4')).patch(() => service4('patch-4'));
app.post('/api/v1/resource-4', () => service4('create-4'));
