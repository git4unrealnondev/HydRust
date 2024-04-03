use crate::database::CacheType;
use log::error;
pub use rusqlite::{params, types::Null, Connection, Result, Transaction};
use std::sync::Arc;
use std::sync::Mutex;
pub trait SqliteConnection {}

pub struct SqliteSearchStructs {
    conn: Arc<Mutex<Connection>>,
    conn_type: CacheType,
}

impl SqliteSearchStructs {
    pub fn new(db_type: CacheType) -> Self {
        let hold = SqliteSearchStructs {
            conn: make_connection(db_type.clone()),
            conn_type: db_type,
        };

        hold
    }
}
fn make_connection(conntype: CacheType) -> Arc<Mutex<Connection>> {
    match conntype {
        CacheType::InMemory => Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
        CacheType::Bare(db_path) => Arc::new(Mutex::new(Connection::open(db_path).unwrap())),
        CacheType::InMemdb => {
            error!("Should not use Inmemdb with SQLITE cache. paniking");
            panic!();
        }
    }
}

//impl DBTraits for SqliteSearchStructs {
impl SqliteSearchStructs {
    fn parents_put(
        &mut self,
        _tag_namespace_id: usize,
        _tag_id: usize,
        _relate_tag_id: usize,
        _relate_namespace_id: usize,
    ) -> usize {
        let _conn = self.conn.lock().unwrap();
        0
    }
}
