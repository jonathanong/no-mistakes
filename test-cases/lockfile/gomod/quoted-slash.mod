module example.invalid/sample

require example.invalid/lib v1.2.3

replace example.invalid/lib => "./local//lib" // local checkout
