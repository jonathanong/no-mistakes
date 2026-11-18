declare const q: any;
declare function run(callback: any): void;
declare function submitForResult(): void;
q["addAndWait"]();
const { waitUntilFinished: wait } = q;
wait();
run(q.addAndWait);
submitForResult();
