declare function inspect(value: string): void;
function helper() { return 'allowed'; }
inspect('allowed');
inspect('forbidden');
inspect(helper());
