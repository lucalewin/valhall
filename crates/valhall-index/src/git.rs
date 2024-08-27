use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use semver::{Version, VersionReq};

use crate::tree::Tree;

use super::{error::Error, models::CrateVersion, IndexTrait};

#[derive(Debug)]
pub struct GitIndex {
    lock: Mutex<()>,
    repo: Repository,
    tree: Tree,
}

impl GitIndex {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            lock: Mutex::new(()),
            repo: Repository {
                path: repo_path.clone(),
            },
            tree: Tree::new(repo_path),
        }
    }
}

impl IndexTrait for GitIndex {
    fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        tracing::debug!("adding record for crate {} ({})", record.name, record.vers);
        let _lock = self.lock.lock();
        // step 0: aquire the lock to block other threads
        //         to commit / push at the same time
        // step 1: create file
        // step 2: commit and push change

        let msg = format!("added crate {} ({:?})", record.name, record.vers);
        self.tree.add_record(record)?;
        self.repo.commit_and_push(&msg)?;
        Ok(())
    }

    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        self.tree.all_records(name)
    }

    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        todo!()
    }

    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        todo!()
    }

    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        todo!()
    }
}

#[derive(Debug)]
pub struct Repository {
    pub path: PathBuf,
}

impl Repository {
    fn url(&self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .stdout(Stdio::piped())
            .current_dir(self.path.canonicalize()?)
            .output()?;

        Ok(String::from_utf8_lossy(output.stdout.as_slice()).into())
    }

    fn refresh(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("pull")
            .arg("--ff-only")
            .current_dir(self.path.canonicalize()?)
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        Command::new("git")
            .arg("add")
            .arg("--all")
            .current_dir(&self.path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(&self.path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("main")
            .current_dir(&self.path)
            .spawn()?
            .wait()?;

        Ok(())
    }
}
