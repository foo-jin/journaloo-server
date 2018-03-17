use chrono::NaiveDateTime;
use db::schema::entries;
use db::models::journey::Journey;
use diesel;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Associations, Serialize, PartialEq, Debug)]
#[table_name = "entries"]
#[belongs_to(Journey)]
pub struct Entry {
    pub id: i32,
    pub journey_id: i32,
    pub created: NaiveDateTime,
    pub archived: bool,
    pub description: Option<String>,
    pub coordinates: Option<String>,
    pub location: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "entries"]
pub struct NewEntry {
    pub journey_id: i32,
    pub description: Option<String>,
    pub coordinates: Option<String>,
    pub location: Option<String>,
}

/// Creates an entry record in the database
pub fn create(entry: &NewEntry, conn: &PgConnection) -> diesel::QueryResult<Entry> {
    use db::schema::entries::dsl::*;
    debug!("creating entry record in db");

    diesel::insert_into(entries)
        .values(entry)
        .get_result::<Entry>(conn)
        .map(|entry| {
            info!("Created entry {:?}", entry);
            entry
        })
        .map_err(|e| {
            error!("Failed to create entry -- {:?}", e);
            e
        })
}

/// Deletes an entry from the database
pub fn archive(entry_id: i32, conn: &PgConnection) -> diesel::QueryResult<()> {
    use db::schema::entries::dsl::*;

    let target = entries.find(entry_id);
    diesel::update(target).set(archived.eq(true)).execute(&*conn)?;
    info!("Archived entry {}", entry_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use db;

    #[test]
    fn create_entry() {
        use super::entries::dsl::*;
        let conn = db::get_test_conn();

        let new_entry = NewEntry {
            journey_id: 2,
            description: Some("asdf".to_string()),
            coordinates: None,
            location: Some("barcelona".to_string()),
        };

        let expected = create(&new_entry, &conn).expect("failed to create entry");
        let result = entries
            .find(expected.id)
            .first::<Entry>(&*conn)
            .expect("error getting result");

        assert_eq!(expected, result);
    }

    #[test]
    fn archive_entry() {
        use super::entries::dsl::*;
        use diesel::NotFound;
        let conn = db::get_test_conn();

        let new_entry = NewEntry {
            journey_id: 1,
            description: None,
            coordinates: None,
            location: None,
        };

        let entry = create(&new_entry, &conn).expect("failed to create entry");
        let eid = entry.id;
        archive(eid, &conn).expect("failed to archive entry");

        match entries.find(eid).first::<Entry>(&*conn) {
            Err(NotFound) => panic!("failed to find entry after archiving"),
            Ok(entry) => assert!(entry.archived),
            Err(e) => panic!("failed to archive entry -- {:?}", e),
        }
    }
}
