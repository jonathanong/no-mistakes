module example.invalid/sample

go 1.25.0
toolchain go1.25.0
godebug asynctimerchan=0

replace (
    example.invalid/library => ../library
)

require example.invalid/library v1.2.3
replace example.invalid/other v1.0.0 => "../shared library"
