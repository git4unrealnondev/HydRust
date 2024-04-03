#![allow(dead_code)]
#![allow(unused_variables)]
use crate::database;
use crate::logging;
use anyhow::Context;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};

use std::sync::{Arc, Mutex};
use std::{
    io::{self, prelude::*, BufReader},
    sync::mpsc::Sender,
};

mod types;

pub fn main(notify: Sender<()>) -> anyhow::Result<()> {
    // Define a function that checks for errors in incoming connections. We'll use this to filter
    // through connections that fail on initialization for one reason or another.
    fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
        match conn {
            Ok(c) => Some(c),
            Err(e) => {
                println!("Incoming connection failed: {}", e);
                None
            }
        }
    }

    // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
    // in Rust, `match` is your concise, readable and expressive decision making construct.
    let name = {
        // This scoping trick allows us to nicely contain the import inside the `match`, so that if
        // any imports of variants named `Both` happen down the line, they won't collide with the
        // enum we're working with here. Maybe someone should make a macro for this.
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => "/tmp/RustHydrus.sock",
            OnlyNamespaced | Both => "@RustHydrus.sock",
        }
    };

    // Bind our listener.
    let listener = match LocalSocketListener::bind(name) {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            // One important problem that is easy to handle improperly (or not at all) is the
            // "corpse sockets" that are left when a program that uses a file-type socket name
            // terminates its socket server without deleting the file. There's no single strategy
            // for handling this kind of address-already-occupied error. Services that are supposed
            // to only exist as a single instance running on a system should check if another
            // instance is actually running, and if not, delete the socket file. In this example,
            // we leave this up to the user, but in a real application, you usually don't want to do
            // that.
            eprintln!(
                "\
Error: could not start server because the socket file is occupied. Please check if {} is in use by \
another process and try again.",
                name,
            );
            return Err(e.into());
        }
        x => x?,
    };

    println!("Server running at {}", name);
    // Stand-in for the syncronization used, if any, between the client and the server.
    let _ = notify.send(());

    // Preemptively allocate a sizeable buffer for reading at a later moment. This size should be
    // enough and should be easy to find for the allocator. Since we only have one concurrent
    // client, there's no need to reallocate the buffer repeatedly.

    for conn in listener.incoming().filter_map(handle_error) {
        let buffer = &mut [b'0', b'0'];
        let mut bufstr = String::new();
        let coms_struct = types::Coms {
            com_type: types::EComType::BiDirectional,
            control: types::EControlSigs::Send,
        };
        let b_struct = bincode::serialize(&coms_struct).unwrap();
        // Wrap the connection into a buffered reader right away
        // so that we could read a single line out of it.
        let mut conn = BufReader::new(conn);
        println!("Incoming connection!");

        // Since our client example writes first, the server should read a line and only then send a
        // response. Otherwise, because reading and writing on a connection cannot be simultaneous
        // without threads or async, we can deadlock the two processes by having both sides wait for
        // the write buffer to be emptied by the other.
        conn.read(buffer).context("Socket receive failed")?;

        // Now that the read has come through and the client is waiting on the server's write, do
        // it. (`.get_mut()` is to get the writer, `BufReader` doesn't implement a pass-through
        // `Write`.)
        //conn.get_mut().write_all(b"Hello from server!\n")?;

        // Print out the result, getting the newline for free!

        let instruct: types::Coms = bincode::deserialize(&buffer[..]).unwrap();
        //std::mem::forget(buffer.clone());

        match instruct.control {
            types::EControlSigs::Send => {
                bufstr.clear();
                conn.read_line(&mut bufstr)
                    .context("Socket receive failed")?;

                //bufstr.clear();

                conn.get_mut()
                    .write_all(&b_struct)
                    .context("Socket send failed")?;
                bufstr.clear();
                conn.read_line(&mut bufstr)
                    .context("Socket receive failed")?;
            }
            types::EControlSigs::Halt => {}
            types::EControlSigs::Break => {}
        }

        // Let's add an exit condition to shut the server down gracefully.
        //if buffer == "stop\n" {
        //    break;
        //}

        // Clear the buffer so that the next iteration will display new data instead of messages
        // stacking on top of one another.
        //buffer.clear();
    }
    Ok(())
}

///
/// Storage for database interaction object for IPC
///
pub struct PluginIpcInteract {
    db_interface: DbInteract,
}

///
/// This is going to be the main way to talk to the plugin system and stuffins.
///
impl PluginIpcInteract {
    pub fn new(main_db: Arc<Mutex<database::Main>>) -> Self {
        PluginIpcInteract {
            db_interface: DbInteract {
                _database: main_db.clone(),
            },
        }
    }

    ///
    /// Spawns a listener for events.
    ///
    pub fn spawn_listener(&mut self, notify: Sender<()>) -> anyhow::Result<()> {
        // Define a function that checks for errors in incoming connections. We'll use this to filter
        // through connections that fail on initialization for one reason or another.
        fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
            match conn {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("Incoming connection failed: {}", e);
                    None
                }
            }
        }

        // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
        // in Rust, `match` is your concise, readable and expressive decision making construct.
        let name = {
            // This scoping trick allows us to nicely contain the import inside the `match`, so that if
            // any imports of variants named `Both` happen down the line, they won't collide with the
            // enum we're working with here. Maybe someone should make a macro for this.
            use NameTypeSupport::*;
            match NameTypeSupport::query() {
                OnlyPaths => "/tmp/RustHydrus.sock",
                OnlyNamespaced | Both => "@RustHydrus.sock",
            }
        };
        // Bind our listener.
        let listener = match LocalSocketListener::bind(name) {
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                // One important problem that is easy to handle improperly (or not at all) is the
                // "corpse sockets" that are left when a program that uses a file-type socket name
                // terminates its socket server without deleting the file. There's no single strategy
                // for handling this kind of address-already-occupied error. Services that are supposed
                // to only exist as a single instance running on a system should check if another
                // instance is actually running, and if not, delete the socket file. In this example,
                // we leave this up to the user, but in a real application, you usually don't want to do
                // that.
                logging::panic_log(&format!(
                    "Error: could not start server because the socket file is occupied. Please check if {} is in use by another process and try again.",name,)
                );
                return Err(e.into());
            }
            x => x?,
        };

        println!("Server running at {}", name);
        // Stand-in for the syncronization used, if any, between the client and the server.
        let _ = notify.send(());

        // Main Plugin interaction loop
        for conn in listener.incoming().filter_map(handle_error) {
            let mut conn = BufReader::new(conn);

            let plugin_supportedrequests = types::recieve(&mut conn);

            //Default

            match plugin_supportedrequests {
                types::SupportedRequests::Database(db_actions) => {
                    let data = self.db_interface.dbactions_to_function(db_actions);

                    types::send_preserialize(&data, &mut conn);
                }
                types::SupportedRequests::PluginCross(_plugindata) => {}
            }
        }
        Ok(())
    }
}

struct DbInteract {
    _database: Arc<Mutex<database::Main>>,
}

///
/// Storage object for database interactions with the plugin system
///
impl DbInteract {
    ///
    /// Helper function to return data about a passed object into size and bytes array.
    ///
    fn data_size_to_b<T: serde::Serialize>(data_object: &T) -> Vec<u8> {
        let tmp = data_object;
        //let bytd = types::x_to_bytes(tmp).to_vec();
        let byt: Vec<u8> = bincode::serialize(&tmp).unwrap();
        byt
    }

    ///
    /// Packages functions from the DB into their self owned versions
    /// before packaging them as bytes to get sent accross IPC to the other
    /// software. So far things are pretty mint.
    ///
    pub fn dbactions_to_function(&mut self, dbaction: types::SupportedDBRequests) -> Vec<u8> {
        match dbaction {
            types::SupportedDBRequests::GetFileLocation(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.get_file(&id);

                Self::option_to_bytes(tmep.as_ref())
            }

            types::SupportedDBRequests::GetFileByte(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.get_file_bytes(&id);
                Self::option_to_bytes(tmep.as_ref())
            }
            types::SupportedDBRequests::Search((search, limit, offset)) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.search_db_files(search, limit, offset);
                Self::data_size_to_b(&tmep)
            }
            types::SupportedDBRequests::GetTagId(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.tag_id_get(&id);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::Logging(log) => {
                logging::info_log(&log);
                Self::data_size_to_b(&true)
            }
            types::SupportedDBRequests::LoggingNoPrint(log) => {
                logging::log(&log);
                Self::data_size_to_b(&true)
            }

            types::SupportedDBRequests::RelationshipAdd(file, tag, addtodb) => {
                let mut unwrappy = self._database.lock().unwrap();
                let _tmep = unwrappy.relationship_add(file, tag, addtodb);
                Self::data_size_to_b(&true)
            }

            types::SupportedDBRequests::PutTag(tags, namespace_id, addtodb, id) => {
                let mut unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.tag_add(tags, namespace_id, addtodb, id);
                Self::data_size_to_b(&tmep)
            }
            types::SupportedDBRequests::GetDBLocation() => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.location_get();
                Self::data_size_to_b(&tmep)
            }
            types::SupportedDBRequests::SettingsSet(name, pretty, num, param, addtodb) => {
                let mut unwrappy = self._database.lock().unwrap();
                unwrappy.setting_add(name, pretty, num, param, addtodb);
                Self::data_size_to_b(&true)
            }

            types::SupportedDBRequests::RelationshipGetTagid(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.relationship_get_tagid(&id);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::GetFile(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.file_get_id(&id);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::RelationshipGetFileid(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.relationship_get_fileid(&id);
                Self::option_to_bytes(tmep)
            }

            types::SupportedDBRequests::SettingsGetName(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.settings_get_name(&id);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::GetTagName((name, namespace)) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.tag_get_name(name, namespace);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::GetFileHash(name) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.file_get_hash(&name);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::CreateNamespace(name, description, addtodb) => {
                let mut unwrappy = self._database.lock().unwrap();
                let out = unwrappy.namespace_add(name, description, addtodb);
                Self::data_size_to_b(&out)
            }

            types::SupportedDBRequests::GetNamespace(name) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.namespace_get(&name);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::TestUsize() => {
                let test: usize = 32;
                Self::data_size_to_b(&test)
            }

            types::SupportedDBRequests::GetNamespaceString(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.namespace_get_string(&id);
                Self::option_to_bytes(tmep)
            }
            types::SupportedDBRequests::LoadTable(table) => {
                let mut unwrappy = self._database.lock().unwrap();
                let _tmep = unwrappy.load_table(&table);
                Self::data_size_to_b(&true)
            }
            types::SupportedDBRequests::TransactionFlush() => {
                let mut unwrappy = self._database.lock().unwrap();
                unwrappy.transaction_flush();
                Self::data_size_to_b(&true)
            }
            types::SupportedDBRequests::GetNamespaceTagIDs(id) => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.namespage_get_tagids(&id);
                Self::data_size_to_b(&tmep)
            }
            types::SupportedDBRequests::GetFileListId() => {
                let unwrappy = self._database.lock().unwrap();
                let tmep = unwrappy.file_get_list_id();
                Self::data_size_to_b(&tmep)
            }
            types::SupportedDBRequests::GetFileListAll() => {
                use std::time::Instant;
                let now = Instant::now();

                let unwrappy = self._database.lock().unwrap();
                let elapsed = now.elapsed();
                println!("Lock Elapsed: {:.2?}", elapsed);

                let tmep = unwrappy.file_get_list_all();
                bincode::serialize(&tmep).unwrap()
            }
        }
    }
    ///
    /// Turns an Option<&T> into a bytes object.
    ///
    fn option_to_bytes<T: serde::Serialize + Clone>(input: Option<&T>) -> Vec<u8> {
        match input {
            None => Self::data_size_to_b(&input),
            Some(item) => {
                let i: Option<T> = Some(item.clone());

                Self::data_size_to_b(&i)
            }
        }
    }
}
