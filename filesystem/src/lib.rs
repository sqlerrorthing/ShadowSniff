#![no_std]

extern crate alloc;
pub mod storage;
pub mod virtualfs;
pub mod path;

use crate::path::Path;
use alloc::vec::Vec;
use core::ops::Deref;

/// Trait representing a generic file system interface.
pub trait FileSystem: AsRef<Self> + Send + Sync {
    /// Reads the entire content of the file at the given path.
    fn read_file<P>(&self, path: P) -> Result<Vec<u8>, u32>
    where
        P: AsRef<Path>;

    /// Creates a single directory at the given path.
    fn mkdir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Creates all missing directories in the given path recursively.
    fn mkdirs<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Removes all contents inside the directory at the given path.
    fn remove_dir_contents<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Removes an empty directory at the given path.
    fn remove_dir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Removes a file at the given path.
    fn remove_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Creates a new empty file at the given path.
    fn create_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Writes data to the file at the given path.
    ///
    /// If the parent directories or the file do not exist, they **should be created**.
    fn write_file<P>(&self, path: P, data: &[u8]) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Lists files under the given directory path, filtered by the provided closure.
    ///
    /// Returns `None` if the directory cannot be listed.
    ///
    /// # Arguments
    ///
    /// * `filter` - A function that takes a file path and returns `true` to include it in the results.
    fn list_files_filtered<F, P>(&self, path: P, filter: &F) -> Option<Vec<Path>>
    where
        F: Fn(&Path) -> bool,
        P: AsRef<Path>;

    /// Returns the creation and modification times of the file at the given path, if available.
    ///
    /// The returned tuple is `(creation_time, modification_time)`.
    fn get_filetime<P>(&self, path: P) -> Option<(u32, u32)>
    where
        P: AsRef<Path>;

    /// Checks if a file or directory exists at the given path.
    fn is_exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>;

    /// Checks if the given path is a directory.
    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>;

    /// Checks if the given path is a file.
    fn is_file<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>;
}

/// Extension trait providing additional helper methods for file systems.
pub trait FileSystemExt: FileSystem {
    /// Removes a directory and all its contents recursively.
    fn remove_dir_all<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>;

    /// Lists all files and directories under the given directory path.
    ///
    /// Returns `None` if the directory cannot be listed.
    fn list_files<P>(&self, path: P) -> Option<Vec<Path>>
    where
        P: AsRef<Path>;
}

impl<F: FileSystem> FileSystemExt for F {
    fn remove_dir_all<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.remove_dir_contents(path)?;
        self.remove_dir(path)
    }

    fn list_files<P>(&self, path: P) -> Option<Vec<Path>>
    where
        P: AsRef<Path>,
    {
        self.list_files_filtered(path, &|_| true)
    }
}

/// Trait for writing data to a file system path.
pub trait WriteTo {
    /// Writes the data to the specified filesystem path.
    fn write_to<F, P>(&self, filesystem: &F, path: P) -> Result<(), u32>
    where
        F: FileSystem,
        P: AsRef<Path>;
}

impl<T> WriteTo for T
where
    T: AsRef<[u8]> + ?Sized,
{
    fn write_to<F, P>(&self, filesystem: &F, path: P) -> Result<(), u32>
    where
        F: FileSystem,
        P: AsRef<Path>,
    {
        filesystem.write_file(path.as_ref(), self.as_ref())
    }
}

/// Copies a single file from `src_fs` at `src_path` to `dst_fs` at `dst_path`.
///
/// If `with_filename` is true, appends the source filename to `dst_path`.
///
/// Creates parent directories in the destination if they do not exist.
pub fn copy_file<SrcRef, SrcFs, SrcPath, DstRef, DstFs, DstPath>(
    src_fs: SrcRef,
    src_path: SrcPath,
    dst_fs: DstRef,
    dst_path: DstPath,
    with_filename: bool,
) -> Result<(), u32>
where
    SrcRef: AsRef<SrcFs>,
    SrcFs: FileSystem,
    DstRef: AsRef<DstFs>,
    DstFs: FileSystem,
    SrcPath: AsRef<Path>,
    DstPath: AsRef<Path>,
{
    let src_fs = src_fs.as_ref();
    let dst_fs = dst_fs.as_ref();
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();

    let dst_path = if with_filename {
        &(dst_path / src_path.fullname().ok_or(2u32)?)
    } else {
        dst_path
    };

    let data = src_fs.read_file(src_path)?;

    if let Some(parent) = dst_path.parent()
        && !dst_fs.is_exists(&parent)
    {
        dst_fs.mkdirs(parent)?;
    }

    dst_fs.write_file(dst_path, &data)
}

/// Copies a folder recursively from `src_fs` at `src_path` to `dst_fs` at `dst_path`,
/// applying a filter function to select which files/directories to copy.
///
/// The copied folder will be created inside `dst_path` using the folder's own name.
pub fn copy_folder_with_filter<
    SrcRef, SrcFs, DstRef, DstFs, SrcPath, DstPath, F
>(
    src_fs: SrcRef,
    src_path: SrcPath,
    dst_fs: DstRef,
    dst_path: DstPath,
    filter: &F,
) -> Result<(), u32>
where
    SrcRef: AsRef<SrcFs>,
    SrcFs: FileSystem,
    DstRef: AsRef<DstFs>,
    DstFs: FileSystem,
    SrcPath: AsRef<Path>,
    DstPath: AsRef<Path>,
    F: Fn(&Path) -> bool,
{
    let src_fs = src_fs.as_ref();
    let dst_fs = dst_fs.as_ref();
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();

    if !src_fs.is_dir(src_path) {
        return Err(1);
    }

    let dst_path = dst_path / src_path.fullname().ok_or(2u32)?;
    copy_content_with_filter(src_fs, src_path, dst_fs, &dst_path, filter)
}

/// Copies a folder recursively from `src_fs` at `src_path` to `dst_fs` at `dst_path`
/// without any filter (copies everything).
pub fn copy_folder<
    SrcRef, SrcFs, SrcPath, DstRef, DstFs, DstPath
>(
    src_fs: SrcRef,
    src_path: SrcPath,
    dst_fs: DstRef,
    dst_path: DstPath,
) -> Result<(), u32>
where
    SrcRef: AsRef<SrcFs>,
    SrcFs: FileSystem,
    DstRef: AsRef<DstFs>,
    DstFs: FileSystem,
    SrcPath: AsRef<Path>,
    DstPath: AsRef<Path>,
{
    copy_folder_with_filter(src_fs, src_path, dst_fs, dst_path, &|_| true)
}

/// Copies all contents of a directory recursively from `src_fs` at `src_path`
/// to `dst_fs` at `dst_path` without filtering.
pub fn copy_content<
    SrcRef, SrcFs, SrcPath, DstRef, DstFs, DstPath
>(
    src_fs: SrcRef,
    src_path: SrcPath,
    dst_fs: DstRef,
    dst_path: DstPath,
) -> Result<(), u32>
where
    SrcRef: AsRef<SrcFs>,
    SrcFs: FileSystem,
    DstRef: AsRef<DstFs>,
    DstFs: FileSystem,
    SrcPath: AsRef<Path>,
    DstPath: AsRef<Path>,
{
    copy_content_with_filter(src_fs, src_path, dst_fs, dst_path, &|_| true)
}

/// Copies the contents of a directory recursively from `src_fs` at `src_path`
/// to `dst_fs` at `dst_path`, applying a filter function.
pub fn copy_content_with_filter<
    SrcRef, SrcFs, SrcPath, DstRef, DstFs, DstPath, F
>(
    src_fs: SrcRef,
    src_path: SrcPath,
    dst_fs: DstRef,
    dst_path: DstPath,
    filter: &F,
) -> Result<(), u32>
where
    SrcRef: AsRef<SrcFs>,
    SrcFs: FileSystem,
    DstRef: AsRef<DstFs>,
    DstFs: FileSystem,
    SrcPath: AsRef<Path>,
    DstPath: AsRef<Path>,
    F: Fn(&Path) -> bool,
{
    let src_fs = src_fs.as_ref();
    let dst_fs = dst_fs.as_ref();
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();

    if !src_fs.is_dir(src_path) {
        return Err(1u32);
    }

    if let Some(files) = &src_fs.list_files_filtered(src_path, filter) {
        for entry in files {
            let relative = entry.strip_prefix(src_path.deref()).ok_or(2u32)?;
            let new_dst = dst_path / relative;

            if src_fs.is_dir(entry) {
                copy_content_with_filter(src_fs, entry, dst_fs, new_dst, filter)?;
            } else {
                copy_file(src_fs, entry, dst_fs, new_dst, false)?;
            }
        }
    }

    Ok(())
}