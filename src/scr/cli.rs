extern crate clap;
use rayon::prelude::*;
use std::path::Path;
use std::{collections::HashSet, io::Write};
//use std::str::pattern::Searcher;
use file_format::FileFormat;
use std::str::FromStr;

use crate::download;
use crate::{
    database, logging, pause, scraper,
    sharedtypes::{self},
};
use clap::Parser;

//use super::sharedtypes::;

use strum::IntoEnumIterator;

mod cli_structs;

///
/// Returns the main argument and parses data.
///
pub fn main(data: &mut database::Main, scraper: &mut scraper::ScraperManager) {
    let args = cli_structs::MainWrapper::parse();

    if let None = &args.a {
        return;
    }

    // Loads settings into DB.
    data.load_table(&sharedtypes::LoadDBTable::Settings);

    match &args.a.as_ref().unwrap() {
        cli_structs::test::Job(jobstruct) => match jobstruct {
            cli_structs::JobStruct::Add(addstruct) => {
                data.load_table(&sharedtypes::LoadDBTable::Jobs);
                let comtype = sharedtypes::CommitType::from_str(&addstruct.committype);
                match comtype {
                    Ok(comfinal) => {
                        let jobs_add = sharedtypes::JobsAdd {
                            site: addstruct.site.to_string(),
                            query: addstruct.query.to_string(),
                            time: addstruct.time.to_string(),
                            committype: comfinal,
                        };

                        data.jobs_add_new(
                            &jobs_add.site,
                            &jobs_add.query,
                            &jobs_add.time,
                            Some(jobs_add.committype),
                            true,
                            sharedtypes::DbJobType::Params,
                        );
                    }
                    Err(_) => {
                        let enum_vec = sharedtypes::CommitType::iter().collect::<Vec<_>>();
                        println!(
                            "Could not parse commit type. Expected one of {:?}",
                            enum_vec
                        );
                        //return sharedtypes::AllFields::Nothing;
                    }
                }
            }
            cli_structs::JobStruct::Remove(_remove) => {
                /*return sharedtypes::AllFields::JobsRemove(sharedtypes::JobsRemove {
                    site: remove.site.to_string(),
                    query: remove.query.to_string(),
                    time: remove.time.to_string(),
                })*/
            }
        },
        cli_structs::test::Search(searchstruct) => match searchstruct {
            cli_structs::SearchStruct::Fid(id) => {
                let hstags = data.relationship_get_tagid(&id.id);
                match hstags {
                    None => {
                        println!(
                            "Cannot find any loaded relationships for fileid: {}",
                            &id.id
                        );
                    }
                    Some(tags) => {
                        for tid in tags.iter() {
                            let tag = data.tag_id_get(tid);
                            match tag {
                                None => {
                                    println!("WANRING CORRUPTION DETECTED for tagid: {}", &tid);
                                }
                                Some(tagnns) => {
                                    let ns = data.namespace_get_string(&tagnns.namespace).unwrap();
                                    println!(
                                        "ID {} Tag: {} namespace: {}",
                                        tid, tagnns.name, ns.name
                                    );
                                }
                            }
                        }
                    }
                }
            }
            cli_structs::SearchStruct::Tid(id) => {
                data.load_table(&sharedtypes::LoadDBTable::All);
                let fids = data.relationship_get_fileid(&id.id);
                if let Some(goodfid) = fids {
                    logging::info_log(&format!("Found Fids:"));
                    for each in goodfid {
                        logging::info_log(&format!("{}", &each));
                    }
                }
            }
            cli_structs::SearchStruct::Tag(tag) => {
                data.load_table(&sharedtypes::LoadDBTable::All);
                let nsid = data.namespace_get(&tag.namespace);
                if let Some(nsid) = nsid {
                    let tid = data.tag_get_name(tag.tag.clone(), *nsid);
                    if let Some(tid) = tid {
                        let fids = data.relationship_get_fileid(tid);
                        if let Some(goodfid) = fids {
                            logging::info_log(&format!("Found Fids:"));
                            for each in goodfid {
                                logging::info_log(&format!("{}", &each));
                            }
                        } else {
                            logging::info_log(&format!(
                                "Cannot find any relationships for tag id: {}",
                                &tid
                            ));
                        }
                    } else {
                        logging::info_log(&format!("Cannot find tag :C"));
                    }
                } else {
                    logging::info_log(&format!("Namespace isn't correct or cannot find it"));
                }
            }
            cli_structs::SearchStruct::Hash(hash) => {
                data.load_table(&sharedtypes::LoadDBTable::All);
                let file_id = data.file_get_hash(&hash.hash);
                match file_id {
                    None => {
                        println!("Cannot find hash in db: {}", &hash.hash);
                    }
                    Some(fid) => {
                        let hstags = data.relationship_get_tagid(fid);
                        match hstags {
                            None => {
                                println!(
                                    "Cannot find any loaded relationships for fileid: {}",
                                    &fid
                                );
                            }
                            Some(tags) => {
                                for tid in tags.iter() {
                                    let tag = data.tag_id_get(tid);
                                    match tag {
                                        None => {
                                            println!(
                                                "WANRING CORRUPTION DETECTED for tagid: {}",
                                                &tid
                                            );
                                        }
                                        Some(tagnns) => {
                                            let ns = data
                                                .namespace_get_string(&tagnns.namespace)
                                                .unwrap();
                                            println!(
                                                "ID {} Tag: {} namespace: {}",
                                                tid, tagnns.name, ns.name
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        cli_structs::test::Tasks(taskstruct) => match taskstruct {
            cli_structs::TasksStruct::Reimport(reimp) => match reimp {
                cli_structs::Reimport::DirectoryLocation(loc) => {
                    if !Path::new(&loc.location).exists() {
                        println!("Couldn't find location: {}", &loc.location);
                        return;
                    }
                    // Loads the scraper info for parsing.
                    let scraperlibrary = scraper.return_libloading_string(&loc.site);
                    let libload = match scraperlibrary {
                        None => {
                            println!("Cannot find a loaded scraper. {}", &loc.site);
                            return;
                        }
                        Some(load) => load,
                    };
                    data.load_table(&sharedtypes::LoadDBTable::Tags);
                    data.load_table(&sharedtypes::LoadDBTable::Files);
                    data.load_table(&sharedtypes::LoadDBTable::Relationship);

                    let failedtoparse: HashSet<String> = HashSet::new();

                    let file_regen = crate::scraper::scraper_file_regen(libload);

                    std::env::set_var("RAYON_NUM_THREADS", "50");

                    println!("Found location: {} Starting to process.", &loc.location);
                    //dbg!(&loc.site, &loc.location);
                    for each in jwalk::WalkDir::new(&loc.location)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|z| z.file_type().is_file())
                    {
                        //println!("{}", each.path().display());
                        //println!("On file: {}", cnt);
                        let (fhist, b) = download::hash_file(
                            &each.path().display().to_string(),
                            &file_regen.hash,
                        );

                        println!("File Hash: {}", &fhist);
                        // Tries to infer the type from the ext.

                        let ext = FileFormat::from_bytes(&b).extension().to_string();
                        // Error handling if we can't parse the filetyp
                        // parses the info into something the we can use for the scraper
                        let scraperinput = sharedtypes::ScraperFileInput {
                            hash: Some(fhist),
                            ext: Some(ext.clone()),
                        };

                        let tag = crate::scraper::scraper_file_return(libload, &scraperinput);
                        // gets sha 256 from the file.
                        let (sha2, _a) = download::hash_bytes(
                            &b,
                            &sharedtypes::HashesSupported::Sha256("".to_string()),
                        );
                        let filesloc = data
                            .settings_get_name(&"FilesLoc".to_string())
                            .unwrap()
                            .param
                            .as_ref()
                            .unwrap()
                            .to_owned();
                        // Adds data into db
                        let fid = data.file_add(None, &sha2, &ext, &filesloc, true);
                        let nid =
                            data.namespace_add(tag.namespace.name, tag.namespace.description, true);
                        let tid = data.tag_add(tag.tag, nid, true, None);
                        data.relationship_add(fid, tid, true);
                        //println!("FIle: {}", each.path().display());
                    }
                    data.transaction_flush();
                    println!("done");
                    if failedtoparse.len() >= 1 {
                        println!("We've got failed items.: {}", failedtoparse.len());
                        for ke in failedtoparse.iter() {
                            println!("{}", ke);
                        }
                    }
                }
            },
            cli_structs::TasksStruct::Database(db) => {
                use crate::helpers;
                use async_std::task;
                match db {
                    cli_structs::Database::BackupDB => {
                        // backs up the db. check the location in setting or code if I change
                        // anything lol
                        data.backup_db();
                    }

                    cli_structs::Database::CheckFiles => {
                        // This will check files in the database and will see if they even exist.
                        let db_location = data.location_get();

                        let cnt: std::sync::Arc<std::sync::Mutex<usize>> =
                            std::sync::Arc::new(std::sync::Mutex::new(0));

                        data.load_table(&sharedtypes::LoadDBTable::All);

                        if !Path::new("fileexists.txt").exists() {
                            let _ = std::fs::File::create("fileexists.txt");
                        }
                        let fiexist: std::sync::Arc<std::sync::Mutex<HashSet<usize>>> =
                            std::sync::Arc::new(std::sync::Mutex::new(
                                std::fs::read_to_string("fileexists.txt")
                                    .unwrap() // panic on possible file-reading errors
                                    .lines() // split the string into an iterator of string slices
                                    .map(|x| x.parse::<usize>().unwrap()) // make each slice into a string
                                    .collect(),
                            ));
                        let f = std::sync::Arc::new(std::sync::Mutex::new(
                            std::fs::File::options()
                                .append(true)
                                .open("fileexists.txt")
                                .unwrap(),
                        ));
                        let lis = data.file_get_list_all();

                        println!("Files do not exist:");
                        let mut nsid: Option<usize> = None;
                        {
                            let nso = data.namespace_get(&"source_url".to_owned());
                            if let Some(ns) = nso {
                                nsid = Some(*ns);
                            }
                        }
                        lis.par_iter().for_each(|each| {
                            if fiexist.lock().unwrap().contains(&each.0) {
                                return;
                            }
                            let loc = helpers::getfinpath(&db_location, &lis[each.0].hash);
                            let lispa = format!("{}/{}", loc, lis[each.0].hash);
                            *cnt.lock().unwrap() += 1;

                            if *cnt.lock().unwrap() == 1000 {
                                let _ = f.lock().unwrap().flush();
                                *cnt.lock().unwrap() = 0;
                            }

                            if !Path::new(&lispa).exists() {
                                println!("{}", &lis[each.0].hash);
                                if nsid.is_some() {
                                    if let Some(rel) = data.relationship_get_tagid(&each.0) {
                                        for eachs in rel {
                                            let dat = data.tag_id_get(eachs).unwrap();
                                            logging::info_log(&format!(
                                                "Got Tag: {} for fileid: {}",
                                                dat.name, each.0
                                            ));
                                            if dat.namespace == nsid.unwrap() {
                                                let client = download::client_create();
                                                let file = &sharedtypes::FileObject {
                                                    source_url: Some(dat.name.clone()),
                                                    hash: Some(
                                                        sharedtypes::HashesSupported::Sha256(
                                                            lis[each.0].hash.clone(),
                                                        ),
                                                    ),
                                                    tag_list: Vec::new(),
                                                };
                                                task::block_on(download::dlfile_new(
                                                    &client,
                                                    file,
                                                    &data.location_get(),
                                                    &mut None,
                                                ));
                                            }
                                        }
                                    }
                                }
                            } else {
                                let fil = std::fs::read(lispa).unwrap();
                                let hinfo = download::hash_bytes(
                                    &bytes::Bytes::from(fil),
                                    &sharedtypes::HashesSupported::Sha256(lis[each.0].hash.clone()),
                                );
                                if !hinfo.1 {
                                    logging::error_log(&format!(
                                        "BAD HASH: ID: {}  HASH: {}   2ND HASH: {}",
                                        &lis[each.0].id.unwrap(),
                                        &lis[each.0].hash,
                                        hinfo.0
                                    ));
                                    if nsid.is_some() {
                                        if let Some(rel) = data.relationship_get_tagid(each.0) {
                                            for eachs in rel {
                                                let dat = data.tag_id_get(eachs).unwrap();
                                                logging::info_log(&format!(
                                                    "Got Tag: {} for fileid: {}",
                                                    dat.name, each.0
                                                ));
                                                if dat.namespace == nsid.unwrap() {
                                                    let client = download::client_create();
                                                    let file = &sharedtypes::FileObject {
                                                        source_url: Some(dat.name.clone()),
                                                        hash: Some(
                                                            sharedtypes::HashesSupported::Sha256(
                                                                lis[each.0].hash.clone(),
                                                            ),
                                                        ),
                                                        tag_list: Vec::new(),
                                                    };
                                                    task::block_on(download::dlfile_new(
                                                        &client,
                                                        file,
                                                        &data.location_get(),
                                                        &mut None,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            fiexist.lock().unwrap().insert(each.0.clone());
                            let fout = format!("{}\n", &each.0).into_bytes();
                            f.lock().unwrap().write_all(&fout).unwrap();
                        });
                        let _ = std::fs::remove_file("fileexists.txt");
                        return;
                    }
                    cli_structs::Database::CheckInMemdb => {
                        data.load_table(&sharedtypes::LoadDBTable::Tags);
                        pause();
                    }

                    cli_structs::Database::CompressDatabase => {
                        data.condese_relationships_tags();
                    }

                    cli_structs::Database::RemoveWhereNot(db_n_rmv) => {
                        let ns_id = match db_n_rmv {
                            cli_structs::NamespaceInfo::NamespaceString(ns) => {
                                data.load_table(&sharedtypes::LoadDBTable::Namespace);
                                let db_id = match data.namespace_get(&ns.namespace_string).cloned()
                                {
                                    None => {
                                        logging::info_log(&format!(
                                            "Cannot find the tasks remove string in namespace {}",
                                            &ns.namespace_string
                                        ));
                                        return;
                                    }
                                    Some(id) => id,
                                };
                                db_id
                            }
                            cli_structs::NamespaceInfo::NamespaceId(ns) => ns.namespace_id,
                        };
                        logging::info_log(&format!(
                            "Found Namespace: {} Removing all but id...",
                            &ns_id
                        ));
                        data.load_table(&sharedtypes::LoadDBTable::Tags);
                        data.load_table(&sharedtypes::LoadDBTable::Relationship);
                        data.load_table(&sharedtypes::LoadDBTable::Parents);
                        //data.namespace_get(inp)

                        let mut key = data.namespace_keys();
                        key.retain(|x| *x != ns_id);
                        for each in key {
                            data.delete_namespace_sql(&each);
                        }

                        data.drop_recreate_ns(&ns_id);

                        panic!();
                    }

                    // Removing db namespace. Will get id to remove then remove it.
                    cli_structs::Database::Remove(db_rmv) => {
                        let ns_id = match db_rmv {
                            cli_structs::NamespaceInfo::NamespaceString(ns) => {
                                data.load_table(&sharedtypes::LoadDBTable::Namespace);
                                let db_id = match data.namespace_get(&ns.namespace_string).cloned()
                                {
                                    None => {
                                        logging::info_log(&format!(
                                            "Cannot find the tasks remove string in namespace {}",
                                            &ns.namespace_string
                                        ));
                                        return;
                                    }
                                    Some(id) => id,
                                };
                                db_id
                            }
                            cli_structs::NamespaceInfo::NamespaceId(ns) => ns.namespace_id,
                        };
                        logging::info_log(&format!("Found Namespace: {} Removing...", &ns_id));
                        data.load_table(&sharedtypes::LoadDBTable::Tags);
                        data.load_table(&sharedtypes::LoadDBTable::Relationship);
                        data.namespace_delete_id(&ns_id);
                    }
                }
            }
            cli_structs::TasksStruct::Csv(_csvstruct) => {}
        },
    }
    //AllFields::Nothing
}
