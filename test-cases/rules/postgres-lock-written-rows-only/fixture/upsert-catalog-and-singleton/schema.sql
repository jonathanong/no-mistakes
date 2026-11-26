CREATE TABLE tasks (id integer PRIMARY KEY, note text);
CREATE TABLE task_parent (id integer PRIMARY KEY, note text);
CREATE TABLE task_child () INHERITS (task_parent);
CREATE TABLE legacy_tasks (id integer PRIMARY KEY, note text);
CREATE TABLE settings (is_singleton boolean PRIMARY KEY, updated_at text DEFAULT 'initial');
