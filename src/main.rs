#![feature(try_trait)]

mod executor;
mod tasks;

use crate::executor::Executor;
use anyhow::anyhow;
use argh::FromArgs;
use log::{info, warn};
use sqlparser::dialect::MySqlDialect;
use sqlparser::parser::Parser;
use std::ops::Try;
use std::{env, fs};
use tasks::SQLStatement;

#[derive(FromArgs)]
#[argh(description = "Execute SQL statements in all possible permutations.")]
struct Args {
    #[argh(option, short = 'c', description = "connect to the database URL")]
    connect: Option<String>,

    #[argh(positional, description = "path to the SQL scripts")]
    sql_scripts: Vec<String>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", concat!(module_path!(), "=debug"));
    }
    env_logger::init();

    let args: Args = argh::from_env();
    let db_url = args.connect.clone().into_result().or_else(|_| {
        warn!("Option --connect not set, using $DATABASE_URL.");
        env::var("DATABASE_URL")
    })?;
    if args.sql_scripts.is_empty() {
        return Err(anyhow!("Expect at least one SQL script"));
    }

    // Parse SQL Scripts. One script corresponds to one client.
    let mut sql_scripts = Vec::with_capacity(args.sql_scripts.len());
    for (i, script) in args.sql_scripts.iter().enumerate() {
        info!("Reading SQLs from {}...", script);
        let sqls = fs::read_to_string(&script)?;

        // NOTE: It's better to use https://github.com/pingcap/parser, but it was written in Go:(
        // TODO: Fully support TiDB syntax.
        let stmts = Parser::parse_sql(&MySqlDialect {}, &sqls)?;
        let raw_sqls = stmts
            .into_iter()
            .map(|s| SQLStatement {
                client_id: i,
                stmt: s.to_string(),
            })
            .collect();
        sql_scripts.push(raw_sqls);
    }

    let tasks = tasks::gen_all_permutations(&sql_scripts);

    let mut executor = Executor::new(&db_url);
    for task in tasks {
        executor.run_task(task).await?;
    }

    Ok(())
}
