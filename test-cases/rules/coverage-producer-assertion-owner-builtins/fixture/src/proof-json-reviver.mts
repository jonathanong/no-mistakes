const callback = () => 1;
JSON.parse('{"value":1}', callback);
JSON.parse('{"value":1}', null);
JSON.parse('{"value":1}');
JSON.stringify({ value: 1 }, callback);
JSON.stringify({ value: 1 }, ['value']);
JSON.stringify({ value: 1 });
