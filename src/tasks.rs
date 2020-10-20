use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SQLStatement {
    /// The Client which will execute the statement.
    pub client_id: usize,
    pub stmt: String,
}

#[derive(Debug, Serialize)]
pub struct Task<'a> {
    id: usize,
    /// The number of clients needed to run the task.
    nb_clients: usize,
    cursor: usize,
    sql_stmts: Vec<&'a SQLStatement>,
}

impl<'a> Iterator for Task<'a> {
    type Item = &'a SQLStatement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.sql_stmts.len() {
            return None;
        }

        let stmt = self.sql_stmts[self.cursor];
        self.cursor += 1;
        Some(stmt)
    }
}

#[allow(dead_code)]
impl<'a> Task<'a> {
    pub(self) fn new(id: usize, nb_clients: usize, sql_stmts: Vec<&'a SQLStatement>) -> Self {
        Self {
            id: id,
            nb_clients: nb_clients,
            cursor: 0,
            sql_stmts: sql_stmts,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn nb_clients(&self) -> usize {
        self.nb_clients
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}

pub fn gen_all_permutations<'a>(sql_scripts: &'a [Vec<SQLStatement>]) -> Vec<Task<'a>> {
    let mut tasks = Vec::new();

    let nb_stmts = sql_scripts.iter().fold(0, |sum, s| sum + s.len());
    let mut stmts = Vec::with_capacity(nb_stmts);

    let mut scripts_ref: Vec<_> = sql_scripts.iter().map(AsRef::as_ref).collect();
    permute(&mut tasks, &mut scripts_ref, &mut stmts);

    /// The core logic of permuting SQL statements.
    #[inline]
    fn permute<'a>(
        tasks: &mut Vec<Task<'a>>,
        scripts: &mut [&'a [SQLStatement]],
        stmts: &mut Vec<&'a SQLStatement>,
    ) {
        if stmts.len() == stmts.capacity() {
            let task = Task::new(tasks.len(), scripts.len(), stmts.to_owned());
            tasks.push(task);
            return;
        }

        for i in 0..scripts.len() {
            let script = scripts[i];
            if script.is_empty() {
                continue;
            }
            stmts.push(&script[0]);
            scripts[i] = &script[1..];
            permute(tasks, scripts, stmts);
            scripts[i] = &script;
            stmts.pop();
        }
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permute() {
        let sql_scripts: Vec<_> = vec![vec![1, 2], vec![3, 6], vec![4, 5]]
            .into_iter()
            .enumerate()
            .map(|(i, s)| {
                s.into_iter()
                    .map(|v| SQLStatement {
                        client_id: i,
                        stmt: v.to_string(),
                    })
                    .collect()
            })
            .collect();

        let tasks = gen_all_permutations(&sql_scripts);
        assert_eq!(90, tasks.len());
    }
}
