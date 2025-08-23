use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use crate::errors::*;
use crate::iter::Iter;

pub struct Finder<'a> {
    filenames: Vec<&'a Path>,
}

impl<'a> Finder<'a> {
    pub fn new() -> Self {
        Finder {
            filenames: vec![Path::new(".env")],
        }
    }

    pub fn filename(mut self, filename: &'a Path) -> Self {
        self.filenames = vec![filename];
        self
    }

    pub fn filenames(mut self, filenames: Vec<&'a Path>) -> Self {
        self.filenames = filenames;
        self
    }

    pub fn find(self) -> Result<(PathBuf, Iter<File>)> {
        if self.filenames.is_empty() {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "No filenames specified",
            )));
        }
        
        let current_dir = &env::current_dir().map_err(Error::Io)?;
        
        for filename in &self.filenames {
            match find(current_dir, filename) {
                Ok(path) => {
                    let file = File::open(&path).map_err(Error::Io)?;
                    let iter = Iter::new(file);
                    return Ok((path, iter));
                }
                Err(e) if e.not_found() => continue,
                Err(e) => return Err(e),
            }
        }
        
        // If we get here, none of the files were found
        Err(Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("None of the specified dotenv files found: {:?}", 
                self.filenames.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()),
        )))
    }
}

/// Searches for `filename` in `directory` and parent directories until found or root is reached.
pub fn find(directory: &Path, filename: &Path) -> Result<PathBuf> {
    let candidate = directory.join(filename);

    match fs::metadata(&candidate) {
        Ok(metadata) => {
            if metadata.is_file() {
                return Ok(candidate);
            }
        }
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                return Err(Error::Io(error));
            }
        }
    }

    if let Some(parent) = directory.parent() {
        find(parent, filename)
    } else {
        Err(Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "path not found",
        )))
    }
}
