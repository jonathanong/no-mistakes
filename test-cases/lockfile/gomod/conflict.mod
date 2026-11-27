module example.invalid/sample

replace (
    <<<<<<< HEAD
    example.invalid/library => ../sample
    =======
    example.invalid/library => ../other
    >>>>>>> topic
)

require example.invalid/library v1.0.0
