import { primary, replica } from '@vendor/valkey';
const box = { client: primary };
box.client = replica;
box.client.get('read');
primary.get('direct');
replica.get('direct');
