//! Locked session directories and atomic, intent-only background snapshots.
use crate::document::schema::Design;
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};
use uuid::Uuid;
#[derive(Clone, Debug)]
pub struct RecoveryCandidate {
    pub path: PathBuf,
    pub label: String,
    pub error: Option<String>,
}
enum Command {
    Write(Vec<(Uuid, Design)>),
    Remove(Uuid),
    Barrier(mpsc::Sender<()>),
}
pub struct RecoveryManager {
    directory: PathBuf,
    _locks: Vec<File>,
    sender: Option<mpsc::Sender<Command>>,
    worker: Option<thread::JoinHandle<()>>,
    results: mpsc::Receiver<(bool, Result<(), String>)>,
    busy: bool,
    pub candidates: Vec<RecoveryCandidate>,
}
fn remove(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Cannot remove recovery {}: {e}", path.display())),
    }
}
impl RecoveryManager {
    pub fn new(root: &Path) -> Result<Self, String> {
        fs::create_dir_all(root)
            .map_err(|e| format!("Cannot create recovery {}: {e}", root.display()))?;
        let mut locks = vec![];
        let mut candidates = vec![];
        for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if !path.is_dir()
                || path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| Uuid::parse_str(n).ok())
                    .is_none()
            {
                continue;
            }
            let Ok(lock) = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(path.join("lock"))
            else {
                continue;
            };
            if lock.try_lock().is_err() {
                continue;
            }
            for file in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let file = file.map_err(|e| e.to_string())?.path();
                if file.extension().is_some_and(|e| e == "con") {
                    let loaded = crate::persistence::container::load(&file);
                    let age = fs::metadata(&file)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|d| format!("{} minutes ago", d.as_secs() / 60))
                        .unwrap_or_else(|| "unknown time".into());
                    let label = match &loaded {
                        Ok(design) => format!(
                            "{} points · {} parameters · snapshot {}",
                            design.points.len(),
                            design.parameters.len(),
                            age
                        ),
                        Err(_) => format!("Damaged snapshot · {}", age),
                    };
                    candidates.push(RecoveryCandidate {
                        path: file,
                        label,
                        error: loaded.err(),
                    });
                }
            }
            locks.push(lock);
        }
        candidates.sort_by(|a, b| a.path.cmp(&b.path));
        let directory = root.join(Uuid::new_v4().to_string());
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(directory.join("lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock().map_err(|e| e.to_string())?;
        let worker_lock = lock.try_clone().map_err(|e| e.to_string())?;
        locks.push(lock);
        let (sender, receiver) = mpsc::channel();
        let (done, results) = mpsc::channel();
        let worker_dir = directory.clone();
        let worker = thread::spawn(move || {
            let _session_lock = worker_lock;
            while let Ok(command) = receiver.recv() {
                match command {
                    Command::Write(snapshots) => {
                        let mut errors = vec![];
                        for (id, design) in snapshots {
                            let path = worker_dir.join(format!("{id}.con"));
                            if let Err(e) = crate::persistence::container::save(&path, &design) {
                                errors.push(format!("Autosave failed at {}: {e}. Check free space and directory permissions", path.display()));
                            }
                        }
                        let _ = done.send((
                            true,
                            if errors.is_empty() {
                                Ok(())
                            } else {
                                Err(errors.join("; "))
                            },
                        ));
                    }
                    Command::Remove(id) => {
                        let _ = done.send((false, remove(&worker_dir.join(format!("{id}.con")))));
                    }
                    Command::Barrier(reply) => {
                        let _ = reply.send(());
                    }
                }
            }
        });
        Ok(Self {
            directory,
            _locks: locks,
            sender: Some(sender),
            worker: Some(worker),
            results,
            busy: false,
            candidates,
        })
    }
    /// At most one batch in flight; callers retain newer state until this returns true.
    pub fn snapshot(&mut self, designs: Vec<(Uuid, Design)>) -> bool {
        if self.busy {
            return false;
        }
        self.busy = self
            .sender
            .as_ref()
            .unwrap()
            .send(Command::Write(designs))
            .is_ok();
        self.busy
    }
    pub fn poll(&mut self) -> Option<Result<(), String>> {
        match self.results.try_recv() {
            Ok((write, result)) => {
                if write {
                    self.busy = false;
                }
                Some(result)
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.busy = false;
                Some(Err(
                    "Recovery worker stopped; restart the application and check recovery storage"
                        .into(),
                ))
            }
            Err(_) => None,
        }
    }
    /// Ordered after queued writes so an old background snapshot cannot resurrect a closed tab.
    pub fn forget(&mut self, id: Uuid) -> Result<(), String> {
        self.sender
            .as_ref()
            .unwrap()
            .send(Command::Remove(id))
            .map_err(|e| e.to_string())
    }
    /// Explicit blocking barrier for headless callers; desktop frame polling never uses this.
    pub fn flush(&self) -> Result<(), String> {
        let (send, receive) = mpsc::channel();
        self.sender
            .as_ref()
            .unwrap()
            .send(Command::Barrier(send))
            .map_err(|e| e.to_string())?;
        receive.recv().map_err(|e| e.to_string())
    }
    pub fn recover(&mut self, index: usize) -> Result<(Uuid, Design), String> {
        let candidate = self
            .candidates
            .get(index)
            .ok_or("Recovery candidate no longer exists")?;
        let design = crate::persistence::container::load(&candidate.path)?;
        let id = Uuid::new_v4();
        fs::rename(&candidate.path, self.directory.join(format!("{id}.con")))
            .map_err(|e| e.to_string())?;
        self.candidates.remove(index);
        Ok((id, design))
    }
    pub fn discard(&mut self, index: usize) -> Result<(), String> {
        let candidate = self
            .candidates
            .get(index)
            .ok_or("Recovery candidate no longer exists")?;
        remove(&candidate.path)?;
        self.candidates.remove(index);
        Ok(())
    }
}

impl Drop for RecoveryManager {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
