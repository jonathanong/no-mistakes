consume(loadFixture("destructured-rebinding").label);
const { loadFixture: get } = helpers;
const id = "destructured-rebinding";
consume(get(id).size);
