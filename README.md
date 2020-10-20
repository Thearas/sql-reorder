# SQL Reorder
Execute SQL statements in all possible permutations.

# Get Start
1.  Setup TiDB (Use TiUP Playground).
```sh
$ ./tests/run_tidb.sh
```

2. Run `sql-reorder`.
```sh
$ export DATABASE_URL="mysql://root@127.0.0.1:4000/mysql"

# create table `X` in `mysql` database
$ cargo run -- tests/sqls/init_table.sql

# permute and execute two SQL scripts
$ cargo run -- tests/sqls/sql1.sql tests/sqls/sql2.sql
```
