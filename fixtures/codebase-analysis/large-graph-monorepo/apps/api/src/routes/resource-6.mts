import app from '../app.mts';
import { service6 } from '../services/service-6.mts';
app.route('/api/v1/resource-6/:id').get(() => service6('read-6')).patch(() => service6('patch-6'));
app.post('/api/v1/resource-6', () => service6('create-6'));
