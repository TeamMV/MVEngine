use bytebuffer::ByteBuffer;
use log::{debug, error, warn};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// A directory that creates itself and children when requested automatically
pub struct SmartDir {
    root: PathBuf,
    exists: AtomicBool,
}

impl Clone for SmartDir {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            exists: AtomicBool::new(self.exists_yet()),
        }
    }
}

impl SmartDir {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = path.as_ref().to_path_buf();
        let exists = fs::exists(path).unwrap_or_default();
        let exists = AtomicBool::new(exists);
        Self { root, exists }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> SmartDir {
        let root = self.root.join(&path);
        let exists = fs::exists(path).unwrap_or_default();
        let exists = AtomicBool::new(exists);
        Self { root, exists }
    }

    pub fn path(&self) -> &PathBuf {
        &self.root
    }

    pub fn exists_yet(&self) -> bool {
        self.exists.load(Ordering::Acquire)
    }

    pub fn exists_file(&self, filename: &str) -> bool {
        let path = self.root.join(filename);
        fs::exists(path).unwrap_or_default()
    }

    pub fn read_file(&self, name: &str) -> Option<File> {
        if !self.exists_yet() {
            if !self.create_self() {
                return None;
            }
        }
        let path = self.root.join(name);
        match File::options().read(true).open(&path) {
            Ok(f) => Some(f),
            Err(e) => {
                debug!("Error when opening file '{path:?}': {e}");
                None
            }
        }
    }

    pub fn write_file(&self, name: &str) -> Option<File> {
        if !self.exists_yet() {
            if !self.create_self() {
                return None;
            }
        }
        let path = self.root.join(name);
        match File::options().write(true).create(true).open(&path) {
            Ok(f) => Some(f),
            Err(e) => {
                error!("Error when opening/creating file '{path:?}': {e}");
                None
            }
        }
    }

    pub fn read_object<T: Savable>(&self, filename: &str) -> Option<T> {
        let mut file = self.read_file(filename)?;
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).ok()?;
        let mut buffer = ByteBuffer::from_vec_le(vec);
        match T::load(&mut buffer) {
            Ok(t) => Some(t),
            Err(e) => {
                warn!(
                    "Could not construct T from {:?}/{}!\n{e}",
                    self.root, filename
                );
                None
            }
        }
    }

    pub fn save_object<T: Savable>(&self, t: &T, filename: &str) -> Option<()> {
        let mut file = self.write_file(filename)?;
        let mut buffer = ByteBuffer::new_le();
        t.save(&mut buffer);
        if let Err(e) = file.write_all(buffer.as_bytes()) {
            warn!("Could not save T to {:?}/{}!\n{e}", self.root, filename);
            None
        } else {
            Some(())
        }
    }

    pub fn create_self(&self) -> bool {
        if let Err(e) = fs::create_dir_all(&self.root) {
            error!("Error when creating SmartDir: {e}");
            false
        } else {
            self.exists.store(true, Ordering::Release);
            true
        }
    }

    pub fn iter_dirs(&self) -> impl Iterator<Item = SmartDir> {
        let root = self.root.clone();
        fs::read_dir(root)
            .into_iter()
            .flatten()
            .filter_map(|x| x.ok()) // skip Errs from read_dir
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    // Construct a SmartDir child
                    Some(self.join(path.file_name()?))
                } else {
                    None
                }
            })
    }

    pub fn dir_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string()
    }

    pub fn delete_child(&self, child_name: &str) {
        let path = self.root.join(child_name);

        let res = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else if path.is_file() {
            fs::remove_file(&path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {:?}", path),
            ))
        };
        if let Err(e) = res {
            warn!(
                "Tried to delete '{child_name}' of '{:?}', but encountered error:\n{e:?}",
                self.root
            );
        }
    }
}

impl Debug for SmartDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}
