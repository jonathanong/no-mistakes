CREATE TABLE c8(a int) USING;
CREATE TABLE c9(a int,b int) PARTITION BY RANGE ();
CREATE TABLE k (id bigint CONSTRAINT k_pk PRIMARY KEY, a int STORAGE plain) USING;
CREATE TABLE k (id bigint CONSTRAINT k_pk PRIMARY KEY, a int STORAGE) USING heap;
CREATE TABLE k (id bigint CONSTRAINT k_pk PRIMARY KEY, a int STORAGE plain) USING heap extra;
CREATE TABLE bad_method_default (a int) USING DEFAULT;
CREATE TABLE bad_storage_method_default (a int STORAGE plain) USING DEFAULT;
