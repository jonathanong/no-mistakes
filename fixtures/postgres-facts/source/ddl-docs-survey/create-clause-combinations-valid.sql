CREATE TABLE k_storage (id bigint CONSTRAINT k_storage_pk PRIMARY KEY, a int STORAGE plain);
CREATE TABLE k_method (id bigint CONSTRAINT k_method_pk PRIMARY KEY, a int) USING heap;
CREATE TABLE k (id bigint CONSTRAINT k_pk PRIMARY KEY, a int STORAGE plain) USING heap;
