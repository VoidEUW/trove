//! Items and kinds: their projection handlers and the service functions the app calls (ADR 5,
//! ADR 6). The tables are a projection of the log (ADR 10): the handlers below are the only code
//! that writes them, and a replay empties and rebuilds them.

use rusqlite::{Connection, OptionalExtension, Row, Transaction};
use serde_json::Value;
use trove_core::{
    Counting, ITEM_ENTITY, Id, Item, ItemDraft, KIND_ENTITY, Kind, KindLabel, Operation,
};

use crate::error::StoreError;
use crate::migrations::SEED_KINDS_SQL;
use crate::projection::Projection;
use crate::store::Store;

fn refuse(message: impl Into<String>) -> StoreError {
    StoreError::Projection(message.into())
}

fn text<'a>(field: &str, value: Option<&'a Value>) -> Result<&'a str, StoreError> {
    value
        .and_then(Value::as_str)
        .ok_or_else(|| refuse(format!("`{field}` must be text and cannot be cleared")))
}

fn whole(field: &str, value: Option<&Value>) -> Result<i64, StoreError> {
    value.and_then(Value::as_i64).ok_or_else(|| {
        refuse(format!(
            "`{field}` must be a whole number and cannot be cleared"
        ))
    })
}

fn flag(field: &str, value: Option<&Value>) -> Result<bool, StoreError> {
    value.and_then(Value::as_bool).ok_or_else(|| {
        refuse(format!(
            "`{field}` must be true or false and cannot be cleared"
        ))
    })
}

/// The new value an operation's patch gives `name`. `None` means the patch leaves it out or clears it.
fn field<'a>(op: &'a Operation, name: &str) -> Option<&'a Value> {
    op.patch
        .iter()
        .find(|(field, _)| *field == name)
        .and_then(|(_, value)| value)
}

fn exists(tx: &Transaction<'_>, table: &str, id: &str) -> Result<bool, StoreError> {
    let sql = format!("SELECT EXISTS (SELECT 1 FROM {table} WHERE id = ?1)");
    Ok(tx.query_row(&sql, [id], |row| row.get(0))?)
}

/// The rows of the `kind` entity. The seed kinds come from the migration; an operation can create
/// or change a kind the user made.
#[derive(Debug)]
pub struct KindProjection;

impl Projection for KindProjection {
    fn entity(&self) -> &str {
        KIND_ENTITY
    }

    fn apply(&self, tx: &Transaction<'_>, op: &Operation) -> Result<(), StoreError> {
        let id = op.entity_id.to_string();
        if !exists(tx, "kinds", &id)? {
            let name = text("name", field(op, "name"))?;
            let counting = text("counting", field(op, "counting"))?;
            if Counting::parse(counting).is_none() {
                return Err(refuse(format!("unknown counting `{counting}`")));
            }
            tx.execute(
                "INSERT INTO kinds (id, name, counting) VALUES (?1, ?2, ?3)",
                (&id, name, counting),
            )?;
        }
        for (field, value) in op.patch.iter() {
            match field {
                "name" => {
                    tx.execute(
                        "UPDATE kinds SET name = ?2, label_key = NULL WHERE id = ?1",
                        (&id, text(field, value)?),
                    )?;
                }
                "counting" => {
                    let counting = text(field, value)?;
                    if Counting::parse(counting).is_none() {
                        return Err(refuse(format!("unknown counting `{counting}`")));
                    }
                    tx.execute(
                        "UPDATE kinds SET counting = ?2 WHERE id = ?1",
                        (&id, counting),
                    )?;
                }
                "container" => {
                    tx.execute(
                        "UPDATE kinds SET container = ?2 WHERE id = ?1",
                        (&id, flag(field, value)?),
                    )?;
                }
                "archived" => {
                    tx.execute(
                        "UPDATE kinds SET archived = ?2 WHERE id = ?1",
                        (&id, flag(field, value)?),
                    )?;
                }
                other => return Err(refuse(format!("unknown kind field `{other}`"))),
            }
        }
        Ok(())
    }

    fn clear(&self, tx: &Transaction<'_>) -> Result<(), StoreError> {
        tx.execute("DELETE FROM kinds", [])?;
        tx.execute_batch(SEED_KINDS_SQL)?;
        Ok(())
    }
}

/// The rows of the `item` entity. Register it after [`KindProjection`]: items refer to kinds.
#[derive(Debug)]
pub struct ItemProjection;

impl Projection for ItemProjection {
    fn entity(&self) -> &str {
        ITEM_ENTITY
    }

    fn apply(&self, tx: &Transaction<'_>, op: &Operation) -> Result<(), StoreError> {
        let id = op.entity_id.to_string();
        let ms = i64::try_from(op.hlc.wall_ms()).map_err(|_| refuse("timestamp out of range"))?;

        // The first operation for an id creates the item, so it must carry everything a row needs.
        if !exists(tx, "items", &id)? {
            tx.execute(
                "INSERT INTO items (id, name, kind_id, quantity, created_ms, updated_ms) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                (
                    &id,
                    text("name", field(op, "name"))?,
                    text("kind", field(op, "kind"))?,
                    whole("quantity", field(op, "quantity"))?,
                    ms,
                ),
            )?;
        }
        for (field, new) in op.patch.iter() {
            match field {
                "name" => tx.execute(
                    "UPDATE items SET name = ?2 WHERE id = ?1",
                    (&id, text(field, new)?),
                )?,
                "kind" => tx.execute(
                    "UPDATE items SET kind_id = ?2 WHERE id = ?1",
                    (&id, text(field, new)?),
                )?,
                "quantity" => tx.execute(
                    "UPDATE items SET quantity = ?2 WHERE id = ?1",
                    (&id, whole(field, new)?),
                )?,
                "archived" => tx.execute(
                    "UPDATE items SET archived = ?2 WHERE id = ?1",
                    (&id, flag(field, new)?),
                )?,
                other => return Err(refuse(format!("unknown item field `{other}`"))),
            };
        }
        tx.execute("UPDATE items SET updated_ms = ?2 WHERE id = ?1", (&id, ms))?;
        Ok(())
    }

    fn clear(&self, tx: &Transaction<'_>) -> Result<(), StoreError> {
        tx.execute("DELETE FROM items", [])?;
        Ok(())
    }
}

fn kind_from(row: &Row<'_>, first: usize) -> Result<Kind, StoreError> {
    let id: String = row.get(first)?;
    let key: Option<String> = row.get(first + 1)?;
    let name: Option<String> = row.get(first + 2)?;
    let counting: String = row.get(first + 3)?;
    let container: bool = row.get(first + 4)?;
    Ok(Kind {
        id: id.parse()?,
        label: match (key, name) {
            (Some(key), _) => KindLabel::Key(key),
            (None, Some(name)) => KindLabel::Text(name),
            (None, None) => return Err(refuse("a kind has no label")),
        },
        counting: Counting::parse(&counting)
            .ok_or_else(|| refuse(format!("unknown counting `{counting}`")))?,
        container,
    })
}

const ITEMS: &str = "SELECT i.id, i.name, i.quantity, i.created_ms, \
                     k.id, k.label_key, k.name, k.counting, k.container \
                     FROM items i JOIN kinds k ON k.id = i.kind_id WHERE i.archived = 0";

fn item_from(row: &Row<'_>) -> Result<Item, StoreError> {
    let id: String = row.get(0)?;
    let created: i64 = row.get(3)?;
    Ok(Item {
        id: id.parse()?,
        name: row.get(1)?,
        quantity: row.get(2)?,
        created_ms: u64::try_from(created).map_err(|_| refuse("negative creation time"))?,
        kind: kind_from(row, 4)?,
    })
}

impl Store {
    /// Validates the draft, then records the item: one `item` operation and its row, in one
    /// transaction. Nothing is stored when the draft is not valid or its kind does not exist.
    pub fn add_item(&self, draft: ItemDraft, now_ms: u64) -> Result<Item, StoreError> {
        let new = draft.validate()?;
        let reader = self.reader()?;
        let known = reader
            .query_row(
                "SELECT 1 FROM kinds WHERE id = ?1 AND archived = 0",
                [new.kind.to_string()],
                |_| Ok(()),
            )
            .optional()?;
        if known.is_none() {
            return Err(StoreError::UnknownKind(new.kind));
        }

        let op = self.record(ITEM_ENTITY, Id::new(now_ms), new.to_patch(), now_ms)?;
        item_by_id(&self.reader()?, op.entity_id)
    }

    /// Every item that is not archived, oldest first.
    pub fn list_items(&self) -> Result<Vec<Item>, StoreError> {
        let reader = self.reader()?;
        let mut statement = reader.prepare(&format!("{ITEMS} ORDER BY i.created_ms, i.rowid"))?;
        let mut rows = statement.query([])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(item_from(row)?);
        }
        Ok(items)
    }

    /// Every kind that is not archived, in the order they were made.
    pub fn list_kinds(&self) -> Result<Vec<Kind>, StoreError> {
        let reader = self.reader()?;
        let mut statement = reader.prepare(
            "SELECT id, label_key, name, counting, container FROM kinds WHERE archived = 0 ORDER BY rowid",
        )?;
        let mut rows = statement.query([])?;
        let mut kinds = Vec::new();
        while let Some(row) = rows.next()? {
            kinds.push(kind_from(row, 0)?);
        }
        Ok(kinds)
    }
}

fn item_by_id(conn: &Connection, id: Id) -> Result<Item, StoreError> {
    let mut statement = conn.prepare(&format!("{ITEMS} AND i.id = ?1"))?;
    let mut rows = statement.query([id.to_string()])?;
    match rows.next()? {
        Some(row) => item_from(row),
        None => Err(refuse(format!(
            "item {id} was not found after it was saved"
        ))),
    }
}
