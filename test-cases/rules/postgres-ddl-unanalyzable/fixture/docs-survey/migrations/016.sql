ALTER SYSTEM SET log_min_duration_statement = -1;
ALTER SYSTEM SET random_page_cost = 1.5;
ALTER SYSTEM SET random_page_cost TO +1.25;
ALTER SYSTEM SET random_page_cost = 1e1;
ALTER DATABASE postgres SET log_min_duration_statement = -1;
ALTER DATABASE postgres SET random_page_cost = .5;
ALTER DATABASE postgres SET random_page_cost TO +1.25;
ALTER DATABASE postgres SET random_page_cost = 1.25e+1;
