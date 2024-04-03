#![allow(dead_code)]
#![allow(unused_variables)]

use crate::sharedtypes::{self};
use anyhow::Context;

use interprocess::local_socket::{LocalSocketStream, NameTypeSupport};
use std::collections::{HashMap, HashSet};
use std::io::BufReader;

pub mod types;
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn search_db_files(
    search: sharedtypes::SearchObj,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Option<HashSet<usize>> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::Search((search, limit, offset)),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn load_table(table: sharedtypes::LoadDBTable) -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::LoadTable(table),
    ))
}
pub fn get_file(fileid: usize) -> Option<String> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetFileLocation(fileid),
    ))
}

pub fn get_file_bytes(fileid: usize) -> Option<Vec<u8>> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetFileByte(fileid),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn namespace_get_string(id: usize) -> Option<sharedtypes::DbNamespaceObj> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetNamespaceString(id),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn log(log: String) -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::Logging(log),
    ))
}
pub fn log_no_print(log: String) -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::LoggingNoPrint(log),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn setting_add(
    name: String,
    pretty: Option<String>,
    num: Option<usize>,
    param: Option<String>,
    addtodb: bool,
) -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::SettingsSet(name, pretty, num, param, addtodb),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn namespace_get(name: String) -> Option<usize> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetNamespace(name),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn namespace_put(name: String, description: Option<String>, addtodb: bool) -> usize {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::CreateNamespace(name, description, addtodb),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn testusize() -> usize {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::TestUsize(),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn transaction_flush() -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::TransactionFlush(),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn relationship_add_db(file: usize, tag: usize, addtodb: bool) -> bool {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::RelationshipAdd(file, tag, addtodb),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn file_get_list_id() -> HashSet<usize> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetFileListId(),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn relationship_get_fileid(id: usize) -> Option<HashSet<usize>> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::RelationshipGetFileid(id),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn tag_get_name(tag: String, namespaceid: usize) -> Option<usize> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetTagName((tag, namespaceid)),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn file_get_list_all() -> HashMap<usize, sharedtypes::DbFileObj> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetFileListAll(),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn settings_get_name(name: String) -> Option<sharedtypes::DbSettingObj> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::SettingsGetName(name),
    ))
}

///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn tag_get_id(id: usize) -> Option<sharedtypes::DbTagNNS> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetTagId(id),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn namespace_get_tagids(id: usize) -> Option<HashSet<usize>> {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetNamespaceTagIDs(id),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn location_get() -> String {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::GetDBLocation(),
    ))
}
///
/// See the database reference for this function.
/// I'm a lazy turd just check it their
///
pub fn tag_add(tag: String, namespace_id: usize, addtodb: bool, id: Option<usize>) -> usize {
    init_data_request(&types::SupportedRequests::Database(
        types::SupportedDBRequests::PutTag(tag, namespace_id, addtodb, id),
    ))
}

///
/// This shouldn't come back to haunt me. :x
/// Returns a Vec of bytes that represent the data structure sent from server.
///
fn init_data_request<T: serde::de::DeserializeOwned>(requesttype: &types::SupportedRequests) -> T {
    let _coms_struct = types::Coms {
        com_type: types::EComType::BiDirectional,
        control: types::EControlSigs::Send,
    };

    let name = {
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => "/tmp/RustHydrus.sock",
            OnlyNamespaced | Both => "@RustHydrus.sock",
        }
    };
    let conn;
    loop {
        // Wait indefinitely for this to get a connection. shit way of doing it will likely add
        // a wait or something this will likely block the CPU or something.
        let temp_conn = LocalSocketStream::connect(name).context("Failed to connect to server");
        if let Ok(con_ok) = temp_conn {
            conn = con_ok;
            break;
        }
    }
    // Wrap it into a buffered reader right away so that we could read a single line out of it.
    let mut conn = BufReader::new(conn);

    // Requesting data from server.
    types::send(requesttype, &mut conn);

    //Recieving size Data from server
    types::recieve(&mut conn)
}
