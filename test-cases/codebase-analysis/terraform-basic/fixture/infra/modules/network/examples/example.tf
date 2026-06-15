# A nested example module. Its directory is NOT a configured module root, so the
# analyzer must not index this file as part of infra/modules/network.
resource "aws_example" "nested" {
  name = "example"
}
