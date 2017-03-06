use chrono;
use postgres;
use r2d2;
use r2d2_postgres::PostgresConnectionManager;

pub type Pool = r2d2::Pool<PostgresConnectionManager>;

#[derive(Debug)]
pub struct Entry {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::UTC>,
}

impl Entry {
    pub fn from_row<'a>(r: postgres::rows::Row<'a>) -> Entry {
        Entry {
            title: r.get(1),
            slug: r.get(2),
            content: r.get(3),
            created_at: r.get(4),
        }
    }
}
