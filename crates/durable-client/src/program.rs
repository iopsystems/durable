use std::borrow::Cow;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::util::LockCell;

pub(crate) type ProgramHash = [u8; 32];

#[derive(Clone, Debug)]
pub struct ProgramOptions {
    pub(crate) name: Option<Cow<'static, str>>,
    pub(crate) wasm: Cow<'static, [u8]>,
}

impl ProgramOptions {
    /// Create a new set of options from the provided WASM binary blob.
    pub fn new(wasm: impl Into<Cow<'static, [u8]>>) -> Self {
        Self {
            wasm: wasm.into(),
            name: None,
        }
    }

    /// Load a program from a file.
    ///
    /// This is a convenience method that reads the file from the provided path
    /// and also sets the name to the last component of the filename.
    ///
    /// # Errors
    /// Returns any errors encountered while reading the file.
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::_from_file(path.as_ref())
    }

    fn _from_file(path: &Path) -> io::Result<Self> {
        let wasm = std::fs::read(path)?;
        let mut opts = Self::new(wasm);

        if let Some(name) = path.file_name() {
            opts = opts.name(name.to_string_lossy().into_owned());
        }

        Ok(opts)
    }

    /// Set a name to be associated with this program.
    ///
    /// This name is a user-readable name that can be viewed at a lated date. It
    /// has no semantic meaning to the runtime.
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct Program(pub(crate) Arc<ProgramData>);

impl Program {
    pub(crate) fn new(data: Arc<ProgramData>) -> Self {
        Self(data)
    }
}

#[derive(Debug)]
pub(crate) struct ProgramData {
    pub(crate) id: AtomicI64,
    pub(crate) hash: ProgramHash,
    pub(crate) wasm: Cow<'static, [u8]>,
    pub(crate) name: Option<Cow<'static, str>>,
    pub(crate) last_used: LockCell<DateTime<Utc>>,
}

impl ProgramData {
    pub fn id(&self) -> i64 {
        self.id.load(Ordering::Acquire)
    }

    pub async fn register(
        hash: ProgramHash,
        wasm: Cow<'static, [u8]>,
        name: Option<Cow<'static, str>>,
        conn: &mut PgConnection,
    ) -> sqlx::Result<Self> {
        let record = sqlx::query!(
            "
            INSERT INTO wasm(hash, wasm, name)
            VALUES ($1, $2, $3)
            ON CONFLICT ON CONSTRAINT hash_unique
            DO UPDATE
            SET last_used = CURRENT_TIMESTAMP
            RETURNING id, last_used
            ",
            hash as ProgramHash,
            &wasm as &[u8],
            name.as_deref()
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(Self {
            id: AtomicI64::new(record.id),
            hash,
            wasm,
            name,
            last_used: LockCell::new(record.last_used),
        })
    }

    pub async fn reregister(&self, conn: &mut PgConnection) -> sqlx::Result<()> {
        let record = sqlx::query!(
            "
            INSERT INTO wasm(hash, wasm, name)
            VALUES ($1, $2, $3)
            ON CONFLICT ON CONSTRAINT hash_unique
            DO UPDATE
            SET last_used = CURRENT_TIMESTAMP
            RETURNING id, last_used
            ",
            self.hash as ProgramHash,
            &self.wasm as &[u8],
            self.name.as_deref()
        )
        .fetch_one(&mut *conn)
        .await?;

        self.id.store(record.id, Ordering::Release);
        self.last_used.set(record.last_used);

        Ok(())
    }
}
