function remove(rows, key) {
  const alias = rows;
  delete alias.value;
  delete rows["value"];
  delete rows[`value`];
  delete rows?.value;
  delete (rows.value as unknown);
  delete rows[key]; // Any member deletion is prohibited; no static key proof is needed.
}
