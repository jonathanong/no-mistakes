declare const field: string;
declare const fields: string[];

ghApi({}, ["repos/fixture/widgets", "-f", "page=1"]);
ghApi({}, ["repos/fixture/widgets", "-f", field]);
ghApi({}, ["repos/fixture/widgets", "-f", `${field}`]);
ghApi({}, ["-F", field, "repos/fixture/widgets"]);
ghApi({}, ["repos/fixture/widgets", "--field", field]);
ghApi({}, ["repos/fixture/widgets", "--raw-field", field]);
ghApi({}, ["repos/fixture/widgets", "-f", field, "-X", "GET"]);
ghApi({}, ["repos/fixture/widgets", "-F", "-X", "GET"]);
ghApi({}, ["repos/fixture/widgets", "-F", ...fields]);
ghApi({}, ["repos/fixture/widgets", "-f"]);
ghApi({}, ["repos/fixture/widgets", "-f", , "-X", "GET"]);
ghApi({}, ["repos/fixture/widgets", "-f", field, "-X", field]);
ghApi({}, ["graphql", "-F", field]);
