use git2::{FetchOptions, RemoteCallbacks, Repository};
use rig::loaders::{file::FileLoaderError, FileLoader};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum GitLoaderError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File loader error: {0}")]
    FileLoaderError(#[from] FileLoaderError),
}

pub struct GitRepo {
    url: String,
    pub(crate) path: PathBuf,
    pub(crate) base_path: PathBuf,
}

impl GitRepo {
    pub fn new(url: String, base_path: PathBuf) -> Self {
        let parts: Vec<&str> = url.trim_end_matches(".git").split('/').collect();
        let (org, repo) = (parts[parts.len() - 2], parts[parts.len() - 1]);
        let path = base_path.join(org).join(repo);
        Self {
            url,
            base_path,
            path,
        }
    }

    pub fn sync(&self) -> Result<Repository, GitLoaderError> {
        if self.path.exists() {
            info!(path = ?self.path, "Repository path exists, updating");
            self.reset()
        } else {
            info!(path = ?self.path, "Repository path does not exist, cloning");
            self.clone()
        }
    }

    fn clone(&self) -> Result<Repository, GitLoaderError> {
        std::fs::create_dir_all(&self.base_path)?;
        debug!(url = %self.url, path = ?self.path, "Cloning repository");
        Ok(Repository::clone(&self.url, &self.path)?)
    }

    fn reset(&self) -> Result<Repository, GitLoaderError> {
        let repo = Repository::open(&self.path)?;

        {
            let mut remote = repo.find_remote("origin")?;
            let callbacks = RemoteCallbacks::new();
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);
            remote.fetch(&["main"], Some(&mut fetch_options), None)?;

            let main_ref = repo.find_reference("refs/remotes/origin/main")?;
            let main_commit = main_ref.peel_to_commit()?;

            let mut checkout_builder = git2::build::CheckoutBuilder::new();

            repo.reset(
                main_commit.as_object(),
                git2::ResetType::Hard,
                Some(&mut checkout_builder),
            )?;
        }

        Ok(repo)
    }
}

pub struct GitLoader<'a> {
    path: &'a str,
    repo: GitRepo,
}

impl<'a> GitLoader<'a> {
    pub fn new(url: String, path: &'a str) -> Result<Self, GitLoaderError> {
        debug!(url = %url, path = path, "Creating new GitLoader");
        let repo = GitRepo::new(url, PathBuf::from(path));
        repo.sync()?;
        Ok(Self { path, repo })
    }

    pub fn with_root(
        self,
    ) -> Result<FileLoader<'a, Result<PathBuf, FileLoaderError>>, FileLoaderError> {
        FileLoader::with_dir(self.path)
    }

    /// Creates a new [FileLoader] using a glob pattern to match files.
    ///
    /// # Example
    /// Create a [FileLoader] for all `.txt` files that match the glob "files/*.txt".
    ///
    /// ```rust
	/// use rig::loaders::FileLoader;
    /// let loader = FileLoader::with_glob("files/*.txt").unwrap();
    /// ```
    pub fn with_glob(
        self,
        pattern: &str,
    ) -> Result<FileLoader<'_, Result<PathBuf, FileLoaderError>>, FileLoaderError> {
        let path = self.repo.path.to_str().unwrap().trim_end_matches('/');
        let pattern = pattern.trim_start_matches('/');
        let glob = Box::leak(format!("{}/{}", path, pattern).into_boxed_str());

        FileLoader::with_glob(glob)
    }

    /// Creates a new [FileLoader] on all files within a directory.
    ///
    /// # Example
    /// Create a [FileLoader] for all files that are in the directory "files" (ignores subdirectories).
    ///
    /// ```rust
	/// use rig::loaders::FileLoader;
    /// let loader = FileLoader::with_dir("files").unwrap();
    /// ```
    pub fn with_dir(
        self,
        directory: &str,
    ) -> Result<FileLoader<'a, Result<PathBuf, FileLoaderError>>, FileLoaderError> {
        let path = Box::leak(
            self.repo
                .path
                .join(directory)
                .to_str()
                .unwrap()
                .to_string()
                .into_boxed_str(),
        );

        FileLoader::with_dir(path)
    }
}
