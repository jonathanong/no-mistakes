CREATE SCHEMA shared;
CREATE TYPE shared.array_priority AS ENUM ('low', 'high');
CREATE TYPE shared.unused_priority AS ENUM ('off', 'on');
CREATE TYPE shared.domain_priority AS ENUM ('off', 'on');
CREATE DOMAIN shared.priority_domain AS shared.domain_priority;
CREATE SCHEMA array_demo;
-- No scalar column uses array_priority: its array alone must retain enum metadata.
CREATE TABLE array_demo.accounts (
  id uuid PRIMARY KEY,
  priorities shared.array_priority[],
  priority shared.priority_domain
);
