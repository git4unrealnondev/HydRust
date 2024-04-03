use libloading::{self, Library};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::{fs, thread};

use crate::database;
use crate::logging;
use crate::sharedtypes;



use crate::server;

pub struct PluginManager {
    _plugin: HashMap<String, libloading::Library>,
    _plugin_coms: HashMap<String, Option<sharedtypes::PluginSharedData>>,
    _callback: HashMap<sharedtypes::PluginCallback, Vec<String>>,
    _database: Arc<Mutex<database::Main>>,
    _thread: HashMap<String, JoinHandle<()>>, // ONLY INSERT INTO ME FOR THE STARTING PLUGIN.
    _thread_path: HashMap<String, String>,    // Will be used for storing path of plugin name.
    _thread_data_share: HashMap<String, (os_pipe::PipeReader, os_pipe::PipeWriter)>,
}

///
/// Plugin Manager Handler
///
impl PluginManager {
    pub fn new(pluginsloc: String, main_db: Arc<Mutex<database::Main>>) -> Self {
        let mut reftoself = PluginManager {
            _plugin: HashMap::new(),
            _callback: HashMap::new(),
            _plugin_coms: HashMap::new(),
            _database: main_db.clone(),
            _thread: HashMap::new(),
            _thread_path: HashMap::new(),
            _thread_data_share: HashMap::new(),
        };

        reftoself.load_plugins(&pluginsloc);

        let (snd, _rcv) = mpsc::channel();
        let _srv = std::thread::spawn(move || {
            let mut ipc_coms = server::PluginIpcInteract::new(main_db.clone());
            //let _ = rcv.recv();
            let out = ipc_coms.spawn_listener(snd);

            //println!("v");
            out
        });
        /* if let Err(e) = srv.join().expect("server thread panicked") {
            eprintln!("Server exited early with error: {:#}", e);
        }

        //let srv = std::thread::spawn(move || server::main(snd));
        if let Err(e) = client::main() {
            eprintln!("Client exited early with error: {:#}", e);
        }*/
        reftoself
    }
    ///
    /// Returns if the thread manager have finished.
    /// Doesn't check if the threads have actually finished.
    ///
    pub fn return_thread(&self) -> bool {
        self._thread.is_empty()
    }

    ///
    /// Closes any threads in self._threads that have finished.
    ///
    pub fn thread_finish_closed(&mut self) {
        let mut finished_threads: Vec<String> = Vec::new();
        let thlist = self._thread.keys();
        for thread in thlist {
            if self._thread.get(thread).unwrap().is_finished() {
                finished_threads.push(thread.to_string());
            }
        }
        for thread in finished_threads {
            let th = self._thread.remove(&thread).unwrap();
            let _ = th.join();
        }
    }

    ///
    /// Loads plugins into plugin manager
    ///
    fn load_plugins(&mut self, pluginsloc: &String) {
        println!("Starting to load plugins at: {}", pluginsloc);

        let ext = ["rlib", "so", "dylib", "dll"];

        let plugin_path = Path::new(pluginsloc);

        // Errors out if I cant create a folder
        if !plugin_path.exists() {
            let path_check = fs::create_dir_all(&plugin_path);
            match path_check {
                Ok(_) => (),
                Err(_) => panic!(
                    "{}",
                    format!("CANNOT CREATE FOLDER: {} DUE TO PERMISSIONS.", &pluginsloc)
                ),
            }
        }

        let dirs = fs::read_dir(&plugin_path).unwrap();

        for entry in dirs {
            let root: String = entry.as_ref().unwrap().path().display().to_string();
            let name = root.split('/');
            let vec: Vec<&str> = name.collect();

            let formatted_name = vec[vec.len() - 1].replace('-', "_");

            let plugin_loading_path = format!(
                "{}{}/target/release/lib{}",
                &pluginsloc,
                vec[vec.len() - 1],
                formatted_name
            );

            let mut finalpath: Option<String> = None;
            'extloop: for exts in ext {
                let testpath = format!("{}.{}", plugin_loading_path, exts);

                // Loading Logic goes here.
                if Path::new(&testpath).exists() {
                    info!("Loading scraper at: {}", &testpath);
                    finalpath = Some(testpath);
                    break 'extloop;
                } else {
                    warn!(
                        "Loading scraper at: {} FAILED due to path not existing",
                        &testpath
                    );
                    finalpath = None;
                }
            }
            if let Some(pathe) = &finalpath {
                let plugininfo: sharedtypes::PluginInfo;
                let lib;
                unsafe {
                    lib = libloading::Library::new(pathe).unwrap();
                    let plugindatafunc: libloading::Symbol<
                        unsafe extern "C" fn() -> sharedtypes::PluginInfo,
                    > = lib.get(b"return_info").unwrap();
                    plugininfo = plugindatafunc();
                }

                let pluginname = plugininfo.name.clone();

                logging::info_log(&format!(
                    "Loaded: {} With Description: {} Plugin Version: {} ABI: {} Comms: {:?}",
                    &pluginname,
                    &plugininfo.description,
                    &plugininfo.version,
                    &plugininfo.api_version,
                    &plugininfo.communication,
                ));

                self._plugin_coms
                    .insert(pluginname.clone(), plugininfo.communication);

                self._plugin.insert(pluginname.clone(), lib);
                self._thread_path
                    .insert(pluginname.clone(), pathe.to_string());

                for each in plugininfo.callbacks {
                    match self._callback.get_mut(&each) {
                        Some(vec_plugin) => {
                            vec_plugin.push(pluginname.clone());
                        }
                        None => {
                            self._callback.insert(each, vec![pluginname.clone()]);
                        }
                    }
                }
            }
        }
    }

    ///
    /// Runs the callback on startup
    ///
    pub fn plugin_on_start(&mut self) {
        // IF theirs no functions with this callback registered then return
        if !self
            ._callback
            .contains_key(&sharedtypes::PluginCallback::OnStart)
        {
            return;
        }
        // Gets all callbacks related to a callback and checks if the plugin
        for plugin in self._callback[&sharedtypes::PluginCallback::OnStart].clone() {
            info!("Starting to run plugin: {}", &plugin);
            if !self._plugin.contains_key(&plugin) {
                error!("Could not call Plugin-OnStart");
                continue;
            }

            // Does a check to see if we need to determine how to pass data to and fro
            match &self._plugin_coms[&plugin] {
                None => {
                    let runloc = self._thread_path[&plugin].to_string();

                    c_run_onstart(&runloc);
                }
                Some(plugincoms) => {
                    match &plugincoms.com_channel {
                        None => {
                            // Starts plugin inline while will wait for it to finish.
                            let runloc = self._thread_path[&plugin].to_string();

                            c_run_onstart(&runloc);
                        }
                        Some(pluginchannel) => {
                            match pluginchannel {
                                sharedtypes::PluginCommunicationChannel::None => {}
                                sharedtypes::PluginCommunicationChannel::Pipe(pipe) => {
                                    dbg!(pipe);

                                    // Have to do this wanky ness to allow me to spawn a thread that outlives the &mut self
                                    // Spawns the function in a seperate thread.
                                    // Have to get this outside of the thread spawn for
                                    // compatibility reasons with the calling funciton.
                                    let runloc = self._thread_path[&plugin].to_string();

                                    let thread = thread::spawn(move || {
                                        c_run_onstart(&runloc);
                                    });
                                    self._thread.insert(plugin.to_string(), thread);
                                }
                            }
                        }
                    }
                    continue;
                }
            }
        }
    }

    ///
    /// Parses output from plugin.
    ///
    fn parse_plugin_output(&mut self, plugin_data: Vec<sharedtypes::DBPluginOutputEnum>) {
        let mut unwrappy = self._database.lock().unwrap();

        for each in plugin_data {
            match each {
                sharedtypes::DBPluginOutputEnum::Add(name) => {
                    for names in name {
                        let mut namespace_id: Option<usize> = Some(0); // holder of namespace

                        // Loops through the namespace objects and selects the last one that's valid.
                        // IF ONE IS NOT VALID THEN THEIR WILL NOT BE ONE ADDED INTO THE DB
                        if let Some(temp) = names.namespace {
                            for namespace in temp {
                                // IF Valid ID && Name && Description info are valid then we have a valid namespace id to pull
                                // dbg!(&namespace);
                                namespace_id = Some(unwrappy.namespace_add(
                                    namespace.name,
                                    namespace.description,
                                    true,
                                ));
                            }
                        }
                        if let Some(temp) = names.file {
                            for files in temp {
                                if files.id.is_none() && files.hash.is_some() && files.ext.is_some()
                                {
                                    if files.location.is_none() {
                                        //let string_dlpath = download::getfinpath(&loc, &files.hash.as_ref().unwrap());
                                        let location = unwrappy
                                            .settings_get_name(&"FilesLoc".to_string())
                                            .unwrap()
                                            .param
                                            .to_owned();
                                        unwrappy.file_add(
                                            None,
                                            &files.hash.unwrap(),
                                            &files.ext.unwrap(),
                                            &location.unwrap(),
                                            true,
                                        );
                                    } else {
                                        unwrappy.file_add(
                                            files.id,
                                            &files.hash.unwrap(),
                                            &files.ext.unwrap(),
                                            &files.location.unwrap(),
                                            true,
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(temp) = names.tag {
                            for tags in temp {
                                let namespace_id = unwrappy.namespace_get(&tags.namespace).cloned();
                                //match namespace_id {}
                                //dbg!(&tags);
                                if tags.parents.is_none() && namespace_id.is_some() {
                                    unwrappy.tag_add(
                                        tags.name.clone(),
                                        namespace_id.unwrap().clone(),
                                        true,
                                        None,
                                    );
                                //                                    println!("plugins323 making tag: {}", tags.name);
                                } else {
                                    for _parents_obj in tags.parents.unwrap() {
                                        unwrappy.tag_add(
                                            tags.name.to_string(),
                                            namespace_id.unwrap().clone(),
                                            true,
                                            None,
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(temp) = names.setting {
                            for settings in temp {
                                unwrappy.setting_add(
                                    settings.name,
                                    settings.pretty,
                                    settings.num,
                                    settings.param,
                                    true,
                                );
                            }
                        }

                        if let Some(_temp) = names.jobs {}
                        if let Some(temp) = names.relationship {
                            for relations in temp {
                                let file_id = unwrappy.file_get_hash(&relations.file_hash).cloned();
                                let namespace_id = unwrappy.namespace_get(&relations.tag_namespace);
                                let tag_id = unwrappy
                                    .tag_get_name(
                                        relations.tag_name.clone(),
                                        namespace_id.unwrap().clone(),
                                    )
                                    .cloned();
                                /*println!(
                                    "plugins356 relating: file id {:?} to {:?}",
                                    file_id, relations.tag_name
                                );*/
                                unwrappy.relationship_add(
                                    file_id.unwrap().clone(),
                                    tag_id.unwrap().clone(),
                                    true,
                                );
                                //unwrappy.relationship_add(file, tag, addtodb)
                            }
                        }
                        if let Some(_temp) = names.parents {}
                    }
                }
                sharedtypes::DBPluginOutputEnum::Del(name) => for _names in name {},
                sharedtypes::DBPluginOutputEnum::None => {}
            }
        }
    }
    //
    /// Runs plugin and
    ///
    pub fn plugin_on_download(&mut self, cursorpass: &[u8], hashs: &String, exts: &String) {
        if !self
            ._callback
            .contains_key(&sharedtypes::PluginCallback::OnDownload)
        {
            return;
        }

        for plugin in self._callback[&sharedtypes::PluginCallback::OnDownload].clone() {
            if !self._plugin.contains_key(&plugin) {
                error!("Could not call Plugin-OnDownload");
                continue;
            }
            let lib = self._plugin.get_mut(&plugin).unwrap();
            let output;
            unsafe {
                let plugindatafunc: libloading::Symbol<
                    unsafe extern "C" fn(
                        &[u8],
                        &String,
                        &String,
                    )
                        -> Vec<sharedtypes::DBPluginOutputEnum>,
                    //unsafe extern "C" fn(Cursor<Bytes>, &String, &String, Arc<Mutex<database::Main>>),
                > = lib.get(b"on_download").unwrap();
                //unwrappy.
                output = plugindatafunc(cursorpass, hashs, exts);
            }
            self.parse_plugin_output(output);
        }
    }
}

///
/// Starts running the onstart plugin.
/// Should only be called from a pluginmanager instance.
/// I'm lazy so this is the easiest way to make it worky.
///
fn c_run_onstart(path: &String) {
    let liba;
    unsafe {
        liba = Library::new(path).unwrap();
    }
    unsafe {
        let plugindatafunc: libloading::Symbol<unsafe extern "C" fn()> =
            liba.get(b"on_start").unwrap();
        plugindatafunc();
    };
}
