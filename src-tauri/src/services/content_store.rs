use std::{
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

pub const INLINE_TEXT_THRESHOLD_BYTES: u64 = 256 * 1024;
pub const CHUNK_SIZE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentStorageKind {
    Inline,
    Chunked,
    FileRef,
}

#[derive(Debug, Clone)]
pub struct ContentStore {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PersistedChunk {
    pub id: String,
    pub chunk_index: u32,
    pub storage_uri: String,
    pub byte_offset: i64,
    pub byte_length: i64,
    pub compression: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportedFile {
    pub primary_storage_uri: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
}

impl ContentStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn ensure_layout(&self) -> crate::support::error::Result<()> {
        fs::create_dir_all(self.blobs_dir())?;
        fs::create_dir_all(self.chunks_dir())?;
        fs::create_dir_all(self.previews_dir())?;
        fs::create_dir_all(self.exports_conversations_dir())?;
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn blobs_dir(&self) -> PathBuf {
        self.root.join("blobs")
    }

    pub fn chunks_dir(&self) -> PathBuf {
        self.root.join("chunks")
    }

    pub fn previews_dir(&self) -> PathBuf {
        self.root.join("previews")
    }

    pub fn exports_dir(&self) -> PathBuf {
        self.root.join("exports")
    }

    pub fn exports_conversations_dir(&self) -> PathBuf {
        self.exports_dir().join("conversations")
    }

    pub fn conversation_export_dir(&self, conversation_id: &str) -> PathBuf {
        self.exports_conversations_dir().join(conversation_id)
    }

    pub fn blob_path(&self, sha256: &str) -> PathBuf {
        let prefix = &sha256[..sha256.len().min(2)];
        self.blobs_dir().join(prefix).join(sha256).join("blob")
    }

    pub fn chunk_path(&self, content_id: &str, chunk_index: u32) -> PathBuf {
        self.chunks_dir()
            .join(content_id)
            .join(format!("{chunk_index:08}.chunk"))
    }

    pub fn choose_storage_kind(
        &self,
        content_type: &str,
        mime_type: Option<&str>,
        size_bytes: u64,
    ) -> ContentStorageKind {
        if Self::prefers_file_ref(content_type, mime_type) {
            return ContentStorageKind::FileRef;
        }

        if size_bytes <= INLINE_TEXT_THRESHOLD_BYTES {
            ContentStorageKind::Inline
        } else {
            ContentStorageKind::Chunked
        }
    }

    pub fn chunk_count(&self, size_bytes: u64) -> u32 {
        if size_bytes == 0 {
            return 0;
        }

        size_bytes.div_ceil(CHUNK_SIZE_BYTES) as u32
    }

    pub fn persist_text_chunks(
        &self,
        content_id: &str,
        bytes: &[u8],
    ) -> crate::support::error::Result<Vec<PersistedChunk>> {
        let mut persisted = Vec::new();

        for (chunk_index, chunk) in bytes.chunks(CHUNK_SIZE_BYTES as usize).enumerate() {
            let path = self.chunk_path(content_id, chunk_index as u32);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, chunk)?;

            persisted.push(PersistedChunk {
                id: crate::support::ids::new_id(),
                chunk_index: chunk_index as u32,
                storage_uri: path.to_string_lossy().to_string(),
                byte_offset: (chunk_index as u64 * CHUNK_SIZE_BYTES) as i64,
                byte_length: chunk.len() as i64,
                compression: None,
                checksum: Some(hex_digest(chunk)),
            });
        }

        Ok(persisted)
    }

    pub fn read_text_chunks(
        &self,
        chunks: &[crate::db::models::ContentChunkRow],
    ) -> crate::support::error::Result<String> {
        let mut buffer = Vec::new();
        for chunk in chunks {
            buffer.extend(fs::read(&chunk.storage_uri)?);
        }

        String::from_utf8(buffer).map_err(|err| {
            crate::support::error::AppError::Validation(format!(
                "chunked content is not valid utf-8: {err}"
            ))
        })
    }

    pub fn import_file(&self, source_path: &Path) -> crate::support::error::Result<ImportedFile> {
        let sha256 = hash_file(source_path)?;
        let destination = self.blob_path(&sha256);

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        if !destination.exists() {
            fs::copy(source_path, &destination)?;
        }

        let size_bytes = fs::metadata(source_path)?.len();

        Ok(ImportedFile {
            primary_storage_uri: destination.to_string_lossy().to_string(),
            size_bytes,
            sha256: Some(sha256),
        })
    }

    fn prefers_file_ref(content_type: &str, mime_type: Option<&str>) -> bool {
        if matches!(
            content_type,
            "image" | "audio" | "video" | "file" | "binary"
        ) {
            return true;
        }

        if let Some(mime) = mime_type {
            return mime.starts_with("image/")
                || mime.starts_with("audio/")
                || mime.starts_with("video/")
                || mime == "application/octet-stream";
        }

        false
    }
}

fn hash_file(path: &Path) -> crate::support::error::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];

    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }

    Ok(format_digest(&hasher.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format_digest(&hasher.finalize())
}

fn format_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
