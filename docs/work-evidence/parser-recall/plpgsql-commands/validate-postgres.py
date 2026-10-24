from pathlib import Path
import getpass,subprocess,tempfile
pg=Path("/usr/lib/postgresql/18/bin")
with tempfile.TemporaryDirectory(prefix="nm-2682-plpgsql-") as tmp:
 root=Path(tmp);data=root/"data";log=open("/tmp/nm-2682-commands-evidence/postgres-valid-fixtures.log","w")
 subprocess.run([str(pg/"initdb"),"-D",str(data),"-A","trust","--no-locale","--encoding=UTF8"],stdout=log,stderr=subprocess.STDOUT,check=True)
 started=False
 try:
  subprocess.run([str(pg/"pg_ctl"),"-D",str(data),"-l",str(root/"server.log"),"-o",f"-k {root} -p 65432 -c listen_addresses=","-w","start"],stdout=log,stderr=subprocess.STDOUT,check=True)
  started=True
  for name in ["valid.sql","declaration-cursor-variants.sql","cursor-sql-facts.sql","dml-query-loops.sql","cursor-first-parameter.sql","grammar-options.sql","with-query-calls.sql"]:
   fixture=Path("test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural/migrations")/name
   log.write("\nFixture: "+str(fixture)+"\n");log.flush()
   subprocess.run([str(pg/"psql"),"-h",str(root),"-p","65432","-U",getpass.getuser(),"-d","postgres","-v","ON_ERROR_STOP=1","-f",str(fixture.resolve())],stdout=log,stderr=subprocess.STDOUT,check=True)
 finally:
  if started: subprocess.run([str(pg/"pg_ctl"),"-D",str(data),"-m","immediate","-w","stop"],stdout=log,stderr=subprocess.STDOUT,check=True)
  log.close()
print("PostgreSQL 18 accepted all seven clean saved PL/pgSQL fixtures")
