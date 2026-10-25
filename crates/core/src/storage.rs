use crate::{
    error::{Error, Result},
    model::{Document, Kit, Setlist, validate_id},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const LIMIT: u64 = 16 * 1024 * 1024;
const JOURNAL: &str = ".kitcontrol-pending.json";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format_version: u32,
    profile_id: String,
    document_id: String,
    kits: Vec<String>,
    slots: Vec<String>,
    setlists: Vec<Setlist>,
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    reject_symlink(path)?;
    let file = File::open(path)?;
    if file.metadata()?.len() > LIMIT {
        return Err(Error::new(
            "file_too_large",
            "JSON file exceeds the 16 MiB limit",
        ));
    }
    Ok(serde_json::from_reader(file)?)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    persist_json(path, value, true)
}

pub fn create_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    persist_json(path, value, false)
}

fn persist_json<T: Serialize>(path: &Path, value: &T, overwrite: bool) -> Result<()> {
    reject_symlink(path)?;
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > LIMIT {
        return Err(Error::new(
            "file_too_large",
            "JSON output exceeds the 16 MiB limit",
        ));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut staging = tempfile::NamedTempFile::new_in(parent)?;
    staging.write_all(&bytes)?;
    staging.as_file().sync_all()?;
    if overwrite {
        staging.persist(path).map_err(|e| Error::from(e.error))?;
    } else {
        staging.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                Error::new(
                    "destination_exists",
                    "Destination exists; choose a new path",
                )
            } else {
                Error::from(e.error)
            }
        })?;
    }
    sync_directory(parent)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    // Unix directory fsync orders renames before removing the recovery marker.
    // Windows has no portable std directory flush; only process-interruption
    // recovery is promised there, not recovery from power loss.
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(Error::new(
            "unsafe_path",
            format!("Refusing a symbolic link: {}", path.display()),
        ));
    }
    Ok(())
}

fn lock(path: &Path, create: bool) -> Result<File> {
    reject_symlink(path)?;
    if create {
        fs::create_dir_all(path)?;
    }
    if !path.is_dir() {
        return Err(Error::new(
            "invalid_workspace",
            "Workspace directory does not exist",
        ));
    }
    let lock_path = path.join(".kitcontrol.lock");
    reject_symlink(&lock_path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    file.try_lock_exclusive().map_err(|_| {
        Error::new(
            "workspace_busy",
            "Another KitControl process owns this workspace",
        )
    })?;
    Ok(file)
}

pub fn load(path: &Path) -> Result<Document> {
    let _lock = lock(path, false)?;
    if path.join(JOURNAL).exists() {
        return Err(Error::new(
            "incomplete_save",
            "An interrupted save is pending. Run recover explicitly before loading.",
        ));
    }
    read_document(path)
}

fn read_document(path: &Path) -> Result<Document> {
    let manifest: Manifest = read_json(&path.join("manifest.json"))?;
    if manifest.format_version != 1 {
        return Err(Error::new(
            "unsupported_version",
            "Only document format version 1 is supported",
        ));
    }
    if manifest.kits.is_empty() || manifest.kits.len() > 1024 {
        return Err(Error::new(
            "invalid_manifest",
            "Manifest must reference 1–1024 kits",
        ));
    }
    reject_symlink(&path.join("kits"))?;
    let mut kits = BTreeMap::new();
    for id in manifest.kits {
        validate_id(&id)?;
        let kit: Kit = read_json(&path.join("kits").join(format!("{id}.json")))?;
        if kits.insert(id, kit).is_some() {
            return Err(Error::new(
                "invalid_manifest",
                "Duplicate kit ID in manifest",
            ));
        }
    }
    let document = Document {
        format_version: manifest.format_version,
        profile_id: manifest.profile_id,
        document_id: manifest.document_id,
        kits,
        slots: manifest.slots,
        setlists: manifest.setlists,
    };
    document.validate()?;
    Ok(document)
}

pub fn save(path: &Path, document: &Document) -> Result<usize> {
    document.validate()?;
    let _lock = lock(path, true)?;
    if path.join(JOURNAL).exists() {
        return Err(Error::new(
            "incomplete_save",
            "Run recover before making another save",
        ));
    }
    if path.join("manifest.json").exists() {
        let old = read_document(path)?;
        if old.document_id != document.document_id || old.profile_id != document.profile_id {
            return Err(Error::new(
                "incompatible_document",
                "Save to a new directory for a different document or profile",
            ));
        }
    } else if fs::read_dir(path)?
        .any(|entry| entry.map_or(true, |e| e.file_name() != ".kitcontrol.lock"))
    {
        return Err(Error::new(
            "unsafe_destination",
            "New workspaces require an empty directory",
        ));
    }
    reject_symlink(&path.join("kits"))?;
    for id in document.kits.keys() {
        reject_symlink(&path.join("kits").join(format!("{id}.json")))?;
    }
    write_json(&path.join(JOURNAL), document)?;
    finish_save(path, document)
}

fn finish_save(path: &Path, document: &Document) -> Result<usize> {
    reject_symlink(&path.join("kits"))?;
    fs::create_dir_all(path.join("kits"))?;
    sync_directory(path)?;
    for (id, kit) in &document.kits {
        write_json(&path.join("kits").join(format!("{id}.json")), kit)?;
    }
    let manifest = Manifest {
        format_version: document.format_version,
        profile_id: document.profile_id.clone(),
        document_id: document.document_id.clone(),
        kits: document.kits.keys().cloned().collect(),
        slots: document.slots.clone(),
        setlists: document.setlists.clone(),
    };
    write_json(&path.join("manifest.json"), &manifest)?;
    // Only remove the application's own completed transaction marker. Kit files
    // are retained, including library kits that no longer occupy a device slot.
    fs::remove_file(path.join(JOURNAL))?;
    sync_directory(path)?;
    Ok(document.kits.len() + 1)
}

pub fn recover(path: &Path) -> Result<usize> {
    let _lock = lock(path, false)?;
    let pending: Document = read_json(&path.join(JOURNAL))?;
    pending.validate()?;
    finish_save(path, &pending)
}

pub fn workspace_path(value: &str) -> Result<PathBuf> {
    if value.trim().is_empty() {
        return Err(Error::new("invalid_path", "Choose a workspace directory"));
    }
    Ok(PathBuf::from(value))
}
