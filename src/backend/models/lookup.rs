use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct Shift {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Colour {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SolventType {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ScrapType {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DowntimeReason {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ManufacturingOrderType {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct FlagReason {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LookupCreatePayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct LookupPayload {
    pub id: i32,
    pub name: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct POCodeSection {
    pub po_code_id: i32,
    pub section_id: i32,
}

#[derive(Deserialize)]
pub struct POCodeSectionPayload {
    pub po_code_id: i32,
    pub section_id: i32,
}

#[derive(Debug, Serialize)]
pub struct POCode {
    pub id: i32,
    pub name: String,
    pub created_at: String,
}



impl Shift {
    pub fn has_related_records(conn: &Connection, shift_id: i32) -> Result<bool> {
        let tables = vec!["jobs", "downtimes", "scraps", "ink_usages", "solvent_usages"];
        for table in tables {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM {} WHERE shift_id = ?1", table),
                params![shift_id],
                |row| row.get(0)
            )?;
            if count > 0 { return Ok(true); }
        }
        Ok(false)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO shifts (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Shift { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE shifts SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM shifts WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM shifts WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Shift { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM shifts ORDER BY name")?;
        let shifts = stmt.query_map([], |row| Ok(Shift { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(shifts)
    }
}

impl Colour {
    pub fn has_related_records(conn: &Connection, colour_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM ink_usages WHERE colour_id = ?1",
            params![colour_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO colours (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Colour { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE colours SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM colours WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM colours WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Colour { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM colours ORDER BY name")?;
        let colours = stmt.query_map([], |row| Ok(Colour { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(colours)
    }
}

impl SolventType {
    pub fn has_related_records(conn: &Connection, solvent_type_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM solvent_usages WHERE solvent_type_id = ?1",
            params![solvent_type_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO solvent_types (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(SolventType { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE solvent_types SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM solvent_types WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM solvent_types WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(SolventType { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM solvent_types ORDER BY name")?;
        let solvent_types = stmt.query_map([], |row| Ok(SolventType { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(solvent_types)
    }
}

impl ScrapType {
    pub fn has_related_records(conn: &Connection, scrap_type_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM scraps WHERE scrap_type_id = ?1",
            params![scrap_type_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO scrap_types (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(ScrapType { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE scrap_types SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM scrap_types WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM scrap_types WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(ScrapType { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM scrap_types ORDER BY name")?;
        let scrap_types = stmt.query_map([], |row| Ok(ScrapType { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(scrap_types)
    }
}

impl ManufacturingOrderType {
    pub fn has_related_records(conn: &Connection, manufacturing_order_type_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM downtimes WHERE manufacturing_order_type_id = ?1",
            params![manufacturing_order_type_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO manufacturing_order_types (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(ManufacturingOrderType { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE manufacturing_order_types SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM manufacturing_order_types WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM manufacturing_order_types WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(ManufacturingOrderType { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM manufacturing_order_types ORDER BY name")?;
        let manufacturing_order_types = stmt.query_map([], |row| Ok(ManufacturingOrderType { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(manufacturing_order_types)
    }
}

impl DowntimeReason {
    pub fn has_related_records(conn: &Connection, downtime_reason_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM downtimes WHERE downtime_reason_id = ?1",
            params![downtime_reason_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO downtime_reasons (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(DowntimeReason { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE downtime_reasons SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM downtime_reasons WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM downtime_reasons WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(DowntimeReason { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM downtime_reasons ORDER BY name")?;
        let downtime_reasons = stmt.query_map([], |row| Ok(DowntimeReason { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(downtime_reasons)
    }
}

impl FlagReason {
    pub fn has_related_records(conn: &Connection, flag_reason_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM rolls WHERE flag_reason_id = ?1",
            params![flag_reason_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }
    
    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        conn.execute("INSERT INTO flag_reasons (name) VALUES (?1)", params![data.name])?;
        let id = conn.last_insert_rowid() as i32;
        Ok(FlagReason { id, name: data.name.clone() })
    }

    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE flag_reasons SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM flag_reasons WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM flag_reasons WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(FlagReason { id: row.get(0)?, name: row.get(1)? }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM flag_reasons ORDER BY name")?;
        let flag_reasons = stmt.query_map([], |row| Ok(FlagReason { id: row.get(0)?, name: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(flag_reasons)
    }
}

impl POCodeSection {
    pub fn create(conn: &Connection, po_code_id: i32, section_id: i32) -> Result<()> {
        conn.execute(
            "INSERT INTO po_code_sections (po_code_id, section_id) VALUES (?1, ?2)",
            params![po_code_id, section_id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, po_code_id: i32, section_id: i32) -> Result<()> {
        conn.execute(
            "DELETE FROM po_code_sections WHERE po_code_id = ?1 AND section_id = ?2",
            params![po_code_id, section_id],
        )?;
        Ok(())
    }

    pub fn find_by_po_code(conn: &Connection, po_code_id: i32) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM po_code_sections WHERE po_code_id = ?1")?;
        let sections = stmt
            .query_map(params![po_code_id], |row| {
                Ok(POCodeSection {
                    po_code_id: row.get(0)?,
                    section_id: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sections)
    }

    pub fn find_by_section(conn: &Connection, section_id: i32) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM po_code_sections WHERE section_id = ?1")?;
        let po_codes = stmt
            .query_map(params![section_id], |row| {
                Ok(POCodeSection {
                    po_code_id: row.get(0)?,
                    section_id: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(po_codes)
    }
}

impl POCode {
    pub fn has_related_records(conn: &Connection, po_code_id: i32) -> Result<bool> {
        let tables = vec!["process_order", "po_code_sections"];
        for table in tables {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM {} WHERE po_code_id = ?1", table),
                params![po_code_id],
                |row| row.get(0)
            )?;
            if count > 0 { return Ok(true); }
        }
        Ok(false)
    }

    pub fn create(conn: &Connection, data: &LookupCreatePayload) -> Result<Self> {
        let now = chrono::Utc::now().naive_utc().to_string();
        conn.execute(
            "INSERT INTO po_codes (name, created_at) VALUES (?1, ?2, ?3)",
            params![data.name, "", now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(POCode { id, name: data.name.clone(), created_at: now })
    }
    
    pub fn update(&mut self, conn: &Connection, data: &LookupPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE po_codes SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM po_codes WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM po_codes WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(POCode {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM po_codes ORDER BY name")?;
        let po_codes = stmt.query_map([], |row| Ok(POCode {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(po_codes)
    }
}

