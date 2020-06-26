use super::data::{Crate, CrateId, Data, Release, Version};
use std::collections::BTreeMap;

pub(crate) fn load(conn: &mut postgres::Client) -> Result<Data, failure::Error> {
    let rows = conn.query(
        "
        SELECT
            crates.name,
            releases.version
        FROM crates
        INNER JOIN releases ON releases.crate_id = crates.id
        ORDER BY crates.id, releases.id
    ",
        &[],
    )?;

    let mut data = Data {
        crates: BTreeMap::new(),
    };

    let mut rows = rows.iter();

    struct Current {
        id: CrateId,
        krate: Crate,
    }

    let mut current = if let Some(row) = rows.next() {
        Current {
            id: CrateId(row.get("name")),
            krate: Crate {
                releases: {
                    let mut releases = BTreeMap::new();
                    releases.insert(Version(row.get("version")), Release {});
                    releases
                },
            },
        }
    } else {
        return Ok(data);
    };

    for row in rows {
        if row.get::<_, String>("name") != current.id.0 {
            data.crates.insert(
                std::mem::replace(&mut current.id, CrateId(row.get("name"))),
                std::mem::take(&mut current.krate),
            );
        }
        current
            .krate
            .releases
            .insert(Version(row.get("version")), Release::default());
    }

    data.crates.insert(current.id, current.krate);

    Ok(data)
}
