CREATE TABLE public.orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE "public.orders" (id int, computed int);
CREATE TABLE "Quoted.Name" (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE "has.""quote" (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE "Orders" (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE "ORDERS" (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
