declare function saveUser(input: string): Promise<void>;

const work = saveUser("x");
work.catch!(() => {});
(work.catch as typeof work.catch)(() => {});
