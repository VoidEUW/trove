//! The open database: version check, backup, connection setup, migration and device id on open, and
//! the service layer that writes operations and replays them (ADR 9, ADR 10).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use rusqlite_migration::{Migrations, SchemaVersion};
use trove_core::{Clock, Id, Operation, Patch};

use crate::error::StoreError;
use crate::projection::Projection;
use crate::{backup, migrations, oplog};

/// The database inside the data folder (ADR 8).
pub const DATABASE_FILE: &str = "trove.db";

/// The folder for snapshots taken before a migration (ADR 8).
pub const BACKUPS_DIR: &str = "backups";

const DEVICE_ID_KEY: &str = "device_id";
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// The open database: one writer connection behind a mutex and read connections on demand.
#[derive(Debug)]
pub struct Store {
    database: PathBuf,
    writer: Mutex<Connection>,
    clock: Mutex<Clock>,
    device_id: Id,
    /// In registration order, so a parent entity can be registered before the ones that refer to it.
    projections: Vec<Box<dyn Projection>>,
}

impl Store {
    /// Opens `trove.db` in the data folder `root`, which the shell resolved. A database left by an
    /// older Trove is backed up and migrated; one left by a newer Trove is refused untouched.
    /// `now_ms` (Unix milliseconds) names the backup and stamps a new device id.
    pub fn open(root: &Path, now_ms: u64) -> Result<Self, StoreError> {
        Self::open_with(root, now_ms, &migrations::all())
    }

    /// [`Store::open`] with an explicit migration list, so upgrades can be tested.
    pub fn open_with(
        root: &Path,
        now_ms: u64,
        migrations: &Migrations<'_>,
    ) -> Result<Self, StoreError> {
        fs::create_dir_all(root).map_err(StoreError::io(root))?;
        let database = root.join(DATABASE_FILE);
        let mut conn = Connection::open(&database)?;

        // Decide before anything is written, so a refused database stays byte for byte as it was.
        let pending = migrations.pending_migrations(&conn)?;
        let existing_version = match migrations.current_version(&conn)? {
            SchemaVersion::NoneSet => None,
            SchemaVersion::Inside(version) => Some(version.get()),
            SchemaVersion::Outside(found) => {
                let supported = found.get().saturating_sub(pending.unsigned_abs() as usize);
                return Err(StoreError::DatabaseTooNew {
                    found: found.get(),
                    supported,
                });
            }
        };

        // A brand-new database has nothing to lose, so only existing data is backed up.
        if let Some(from) = existing_version
            && pending > 0
        {
            backup::before_migration(&conn, &root.join(BACKUPS_DIR), from, now_ms)?;
        }

        configure(&conn)?;
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
        migrations.to_latest(&mut conn)?;
        let device_id = device_id(&mut conn, now_ms)?;
        let clock = Clock::new(oplog::latest_hlc(&conn)?);

        Ok(Self {
            database,
            writer: Mutex::new(conn),
            clock: Mutex::new(clock),
            device_id,
            projections: Vec::new(),
        })
    }

    /// Registers the handler for one entity type. Register at startup, parents before the entities
    /// that refer to them: replay empties the tables in the opposite order.
    pub fn register(&mut self, projection: impl Projection + 'static) -> Result<(), StoreError> {
        let entity = projection.entity();
        if self.projection(entity).is_ok() {
            return Err(StoreError::Projection(format!(
                "entity `{entity}` already has a projection"
            )));
        }
        self.projections.push(Box::new(projection));
        Ok(())
    }

    /// Records a local change: stamps it with this device's next clock value, appends it to the log
    /// and applies it to the projection in one transaction. If either step fails, neither is stored
    /// and the clock does not move. `entity_id` is a new id for a new entity, so the first call for
    /// an id creates it.
    pub fn record(
        &self,
        entity: &str,
        entity_id: Id,
        patch: Patch,
        now_ms: u64,
    ) -> Result<Operation, StoreError> {
        self.with_writer(|conn| {
            let mut clock = self.clock.lock().unwrap_or_else(PoisonError::into_inner);
            let mut next = *clock;
            let hlc = next.tick(now_ms)?;
            let op = Operation::new(self.device_id, hlc, entity, entity_id, patch);
            self.apply_in(conn, &op)?;
            *clock = next;
            Ok(op)
        })
    }

    /// Appends `op` to the log and applies it to the projection in one transaction: if the log
    /// refuses it (an id already present) or the entity's handler fails, neither is stored.
    pub fn apply(&self, op: &Operation) -> Result<(), StoreError> {
        self.with_writer(|conn| self.apply_in(conn, op))
    }

    /// Rebuilds the projection from the log: empties every registered handler's tables, then applies
    /// all operations in `(hlc, device_id)` order. All or nothing. Returns how many were applied.
    pub fn replay(&self) -> Result<usize, StoreError> {
        self.with_writer(|conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            for projection in self.projections.iter().rev() {
                projection.clear(&tx)?;
            }
            let mut applied = 0;
            {
                let mut statement = tx.prepare(oplog::ORDERED)?;
                let mut rows = statement.query([])?;
                while let Some(row) = rows.next()? {
                    let op = oplog::operation_from_row(row)?;
                    self.projection(&op.entity)?.apply(&tx, &op)?;
                    applied += 1;
                }
            }
            tx.commit()?;
            Ok(applied)
        })
    }

    /// This installation's id, created on first open and the same ever after.
    pub fn device_id(&self) -> Id {
        self.device_id
    }

    pub fn database(&self) -> &Path {
        &self.database
    }

    /// Runs `f` on the single writer connection. Only the store's service layer writes (ADR 9).
    pub fn with_writer<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        // A panic inside an earlier `f` left the connection in a consistent state: SQLite rolled
        // its open transaction back when the guard dropped.
        let mut writer = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        f(&mut writer)
    }

    fn apply_in(&self, conn: &mut Connection, op: &Operation) -> Result<(), StoreError> {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        self.apply_to(&tx, op)?;
        tx.commit()?;
        Ok(())
    }

    fn apply_to(&self, tx: &Transaction<'_>, op: &Operation) -> Result<(), StoreError> {
        oplog::check_version(op)?;
        let projection = self.projection(&op.entity)?;
        oplog::append(tx, op)?;
        projection.apply(tx, op)
    }

    fn projection(&self, entity: &str) -> Result<&dyn Projection, StoreError> {
        self.projections
            .iter()
            .find(|projection| projection.entity() == entity)
            .map(|projection| &**projection)
            .ok_or_else(|| StoreError::UnknownEntity(entity.to_owned()))
    }

    /// A new read-only connection. WAL lets any number of them read while the writer writes.
    pub fn reader(&self) -> Result<Connection, StoreError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(&self.database, flags)?;
        configure(&conn)?;
        Ok(conn)
    }
}

/// Settings every connection needs.
fn configure(conn: &Connection) -> Result<(), StoreError> {
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    Ok(())
}

/// Reads the device id from `meta`, creating it on the very first open.
fn device_id(conn: &mut Connection, now_ms: u64) -> Result<Id, StoreError> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT OR IGNORE INTO meta (key, value) VALUES (?1, ?2)",
        (DEVICE_ID_KEY, Id::new(now_ms).to_string()),
    )?;
    let stored: String = tx.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        [DEVICE_ID_KEY],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(stored.parse()?)
}
