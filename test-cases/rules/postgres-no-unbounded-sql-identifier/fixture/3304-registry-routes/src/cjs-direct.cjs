const direct = require("./catalog"); const picked = require("./catalog").TABLES; const alias = picked; const returned = (() => require("./catalog"))();
