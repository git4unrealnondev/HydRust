#![allow(dead_code)]
#![allow(unused_variables)]
use crate::sharedtypes;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub enum EComType {
    SendOnly,
    RecieveOnly,
    BiDirectional,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EControlSigs {
    Send,  // Sending data to and fro
    Halt,  // Come to a stop naturally
    Break, // STOP NOW PANIC
}

///
/// Main communication block structure.
///

#[derive(Debug, Serialize, Deserialize)]
pub struct Coms {
    pub com_type: EComType,
    pub control: EControlSigs,
}
/*pub fn x_to_bytes<T: Sized>(input_generic: &T) -> &[u8] {
    unsafe { any_as_u8_slice(input_generic) }
}

///
/// Turns a generic into bytes
///
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

///
/// Turns bytes into a coms structure.
///
pub fn con_coms(input: &mut [u8; 2]) -> Coms {
    unsafe { std::mem::transmute(*input) }
}

///
/// Turns bytes into a controlsig structure.
///
pub fn con_econtrolsigs(input: &mut [u8; 1]) -> EControlSigs {
    unsafe { std::mem::transmute(*input) }
}

///
/// Turns bytes into a uszie structure.
///
pub fn con_usize(input: &mut [u8; 8]) -> usize {
    unsafe { std::mem::transmute(*input) }
}

///
/// Turns bytes into a SupportedRequests structure.
///
//pub fn con_supportedrequests(input: &mut [u8; 56]) -> SupportedRequests {
//    unsafe { std::mem::transmute(*input) }
//}
*/
///
/// Supported Database operations.
///
#[derive(Debug, Serialize, Deserialize)]
pub enum SupportedDBRequests {
    GetTagId(usize),
    PutTag(String, usize, bool, Option<usize>),
    GetTagName((String, usize)),
    RelationshipAdd(usize, usize, bool),
    RelationshipGetTagid(usize),
    RelationshipGetFileid(usize),
    GetFile(usize),
    GetFileHash(String),
    GetNamespace(String),
    CreateNamespace(String, Option<String>, bool),
    GetNamespaceTagIDs(usize),
    GetNamespaceString(usize),
    SettingsGetName(String),
    SettingsSet(String, Option<String>, Option<usize>, Option<String>, bool),
    LoadTable(sharedtypes::LoadDBTable),
    TestUsize(),
    GetFileListId(),
    GetFileListAll(),
    TransactionFlush(),
    GetDBLocation(),
    Logging(String),
    LoggingNoPrint(String),
    Search((sharedtypes::SearchObj, Option<usize>, Option<usize>)),
    GetFileByte(usize),
    GetFileLocation(usize),
}

///
/// Actions for Database
///

///
/// Returns all data, general structure.
///
#[derive(Debug)]
pub enum AllReturns {
    DB(DBReturns),
    Plugin(SupportedPluginRequests),
    Nune, // Placeholder don't actually use. I'm using it lazizly because I'm a shitter. Keep it
          // here for handling edge cases or nothing needs to get sent. lol
}

///
/// Returns the db data
///
#[derive(Debug)]
pub enum DBReturns {
    GetTagId(Option<sharedtypes::DbTagNNS>),
    GetTagName(Option<usize>),
    RelationshipGetTagid(Option<HashSet<usize>>),
    RelationshipGetFileid(Option<HashSet<usize>>),
    GetFile(Option<sharedtypes::DbFileObj>),
    GetFileHash(Option<usize>),
    GetNamespaceTagIDs(HashSet<usize>),
    GetNamespace(Option<usize>),
    GetNamespaceString(Option<sharedtypes::DbNamespaceObj>),
    SettingsGetName(Option<sharedtypes::DbSettingObj>),
    LoadTable(bool),
}

pub enum EfficientDataReturn {
    Data(Vec<u8>),
    Nothing,
}

///
/// Supported cross talks between plugins.
///
#[derive(Debug, Deserialize, Serialize)]
pub enum SupportedPluginRequests {}

///
/// Supported enum requests for the transaction.
/// Will get sent over to sever / client to determine what data will be sent back and forth.
///
#[derive(Debug, Deserialize, Serialize)]
pub enum SupportedRequests {
    Database(SupportedDBRequests),
    PluginCross(SupportedPluginRequests),
}

#[derive(Debug, Deserialize, Serialize)]
struct Effdata {
    #[serde(with = "serde_bytes")]
    byte_buf: Vec<u8>,
}

///
/// Writes all data into buffer.
///
pub fn send<T: Sized + Serialize>(
    inp: T,
    conn: &mut BufReader<interprocess::local_socket::LocalSocketStream>,
) {
    let byte_buf = bincode::serialize(&inp).unwrap();
    let size = &byte_buf.len();
    conn.get_mut()
        .write_all(&size.to_ne_bytes())
        .context("Socket send failed")
        .unwrap();
    conn.get_mut()
        .write_all(&byte_buf)
        .context("Socket send failed")
        .unwrap();
}
///
/// Writes all data into buffer.
/// Assumes data is preserialzied from data generic function.
/// Can be hella dangerous. Types going in and recieved have to match EXACTLY.
///
pub fn send_preserialize(
    inp: &Vec<u8>,
    conn: &mut BufReader<interprocess::local_socket::LocalSocketStream>,
) {
    let mut temp = inp.len().to_ne_bytes().to_vec();
    temp.extend(inp);
    let _ = conn
        .get_mut()
        .write_all(&temp)
        .context("Socket send failed");

    return;
}

///
/// Returns a vec of bytes that represent an object
///
pub fn recieve<T: serde::de::DeserializeOwned>(
    conn: &mut BufReader<interprocess::local_socket::LocalSocketStream>,
) -> T {
    let mut usize_b: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<usize>()];
    conn.get_mut()
        .read_exact(&mut usize_b[..])
        .context("Socket send failed")
        .unwrap();

    let size_of_data: usize = usize::from_ne_bytes(usize_b);

    let mut data_b = vec![0; size_of_data];
    conn.get_mut()
        .read_exact(&mut data_b[..])
        .context("Socket send failed")
        .unwrap();
    bincode::deserialize(&data_b).unwrap()
}
