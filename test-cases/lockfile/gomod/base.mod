module example.com/sample

go 1.22

require (
	example.com/sample-alpha v1.2.3
	example.com/sample-beta v0.4.0 // indirect
)

require example.com/sample-gamma v1.0.0

exclude example.com/sample-alpha v1.0.0

replace example.com/sample-beta => ../sample-beta
