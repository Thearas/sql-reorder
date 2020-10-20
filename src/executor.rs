use crate::tasks::{SQLStatement, Task};
use anyhow::anyhow;
use log::{debug, error, trace};
use sqlx::mysql::MySqlDone;
use sqlx::{MySql, Pool};

/// An Executor holds multiple [`TiDBClient`]s, and runs [`Task`] on them.
///
/// [`TiDBClient`]: TiDBClient
/// [`Task`]: crate::tasks::Task
pub struct Executor {
    // It's better to use server discovery.
    db_url: String,
    clients: Vec<TiDBClient>,
    exit_on_fail: bool,
}

#[derive(Debug)]
pub struct TiDBClient {
    db_url: String,
    pool: Pool<MySql>,
}

impl Executor {
    pub fn new(db_url: &str) -> Self {
        Self {
            db_url: db_url.to_owned(),
            clients: Vec::new(),
            exit_on_fail: true,
        }
    }

    pub async fn run_task<'a>(&mut self, task: Task<'a>) -> anyhow::Result<()> {
        // Auto scaling when clients are not enough.
        self.reserve_clients(task.nb_clients()).await?;

        let tid = task.id();
        for (i, sql) in task.enumerate() {
            let SQLStatement { client_id, stmt } = sql;
            let client = self.clients.get(*client_id).ok_or_else(|| {
                anyhow!(
                    "client not enough, expect: {}, but only has: {}",
                    client_id,
                    self.clients.len(),
                )
            })?;

            let log_tag = format!("[Task {},{}][Cli {}]", tid, i, client_id);
            trace!("{} executing: {}", log_tag, stmt);

            match client.execute(&stmt).await {
                Ok(done) => debug!("{} done: {:?}", log_tag, done),
                Err(e) => {
                    error!("{} error while executing {:?}, err: {:?}", log_tag, stmt, e);
                    if self.exit_on_fail {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn reserve_clients(&mut self, need: usize) -> anyhow::Result<()> {
        let curr = self.clients.len();
        if need > curr {
            for i in curr..need {
                let client = TiDBClient::connect(i, &self.db_url).await?;
                self.clients.push(client);
            }
        }

        Ok(())
    }
}

impl TiDBClient {
    pub async fn connect(id: usize, url: &str) -> anyhow::Result<Self> {
        debug!("[Client {}] Connecting to: {}...", id, url);
        let pool = sqlx::MySqlPool::connect(url).await?;
        Ok(Self {
            db_url: url.to_owned(),
            pool: pool,
        })
    }

    pub async fn execute(&self, sql: &str) -> anyhow::Result<MySqlDone> {
        Ok(sqlx::query(sql).execute(&self.pool).await?)
    }
}

// impl Deref for TiDBClient {
//     type Target = Pool<MySql>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }
//
// impl DerefMut for TiDBClient {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }
