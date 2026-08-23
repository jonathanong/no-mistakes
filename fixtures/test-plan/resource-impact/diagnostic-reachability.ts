import fs from 'node:fs';

const dynamicPath = process.env.RESOURCE_PATH;

const api = {
  load() {
    fs.readFileSync(dynamicPath);
  },
};

api.load();
