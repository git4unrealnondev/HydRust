#![allow(dead_code)]
#![allow(unused_variables)]

use super::database;
use super::download;
use super::sharedtypes;
use crate::helpers;
use ahash::AHashMap;
use csv;
use file_format::FileFormat;
use log::{error, info};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Row {
    path: String,
    tag: String,
    namespace: String,
    parent: Option<String>,
    id: usize,
}

///
/// Just a holder for tasks. Can be called from here or any place really. :D
/// Currently supports only one file to tag assiciation.
/// Need to add support for multiple tags. But this currently works for me.
///
pub fn import_files(
    location: &String,
    csvdata: sharedtypes::CsvCopyMvHard,
    db: &mut database::Main,
) {
    if !Path::new(&location).exists() {
        error!("Path: {} Doesn't exist. Exiting. Check logs", &location);
        panic!("Path: {} Doesn't exist. Exiting. Check logs", &location);
    }

    let mut rdr = csv::ReaderBuilder::new().from_path(location).unwrap();

    let mut headers: Vec<String> = Vec::new();
    let headerrecord = rdr.headers().unwrap().clone();
    for head in headerrecord.iter() {
        headers.push(head.to_string());
    }

    // Checks if path is missing
    if !headers.contains(&"path".to_string()) {
        error!("CSV ERROR, issue with csv file. No path header.");
        panic!("CSV ERROR, issue with csv file. No path header.");
    }

    let location = db
        .settings_get_name(&"FilesLoc".to_string())
        .unwrap()
        .param
        .as_ref()
        .unwrap()
        .to_owned();

    println!("Importing Files to: {}", &location);

    let mut delfiles: AHashMap<String, String> = AHashMap::new();

    for line in rdr.records() {
        let row: Row = line
            .as_ref()
            .unwrap()
            .deserialize(Some(&headerrecord))
            .unwrap();

        if !Path::new(&row.path).exists() {
            error!("Path: {} Doesn't exist. Exiting. Check logs", &row.path);
            println!("Path: {} Doesn't exist. Exiting. Check logs", &row.path);
            continue;
        }
        let (hash, _b) = download::hash_file(
            &row.path,
            &sharedtypes::HashesSupported::Sha256("".to_string()),
        );

        let hash_exists = db.file_get_hash(&hash);

        if hash_exists.is_some() {
            //delfiles.insert(row.path.to_string(), "".to_owned());
            fs::remove_file(&row.path).unwrap(); // Removes file that's already in DB.
            println!("File: {} already in DB. Skipping import.", &row.path);
            info!("File: {} already in DB. Skipping import.", &row.path);
            continue;
        }

        let path = helpers::getfinpath(&location, &hash);

        let final_path = format!("{}/{}", path, &hash);

        let file_ext = FileFormat::from_file(&row.path)
            .unwrap()
            .extension()
            .to_string();

        // Completes file actions.
        match csvdata {
            sharedtypes::CsvCopyMvHard::Copy => {
                fs::copy(&row.path, &final_path).unwrap();
            }
            sharedtypes::CsvCopyMvHard::Move => {
                fs::copy(&row.path, &final_path).unwrap();
                delfiles.insert(row.path.to_string(), "".to_owned());
            }
            sharedtypes::CsvCopyMvHard::Hardlink => {
                fs::hard_link(&row.path, &final_path).unwrap();
            }
        }
        println!("Copied to path: {}", &final_path);

        dbg!(&hash, &row);
        // Adds into DB
        let file_id = db.file_add(None, &hash, &file_ext, &location, true);
        let namespace_id = db.namespace_add(row.namespace, None, true);
        let tag_id = db.tag_add(row.tag.to_string(), namespace_id, true, Some(row.id));

        db.relationship_add(file_id.to_owned(), tag_id.to_owned(), true);
    }
    db.transaction_flush();
    println!("Clearing any files from any move ops.");
    info!("Clearing any files from any move ops.");
    for each in delfiles.keys() {
        fs::remove_file(each).unwrap();
    }
    dbg!("Done!");
    info!("Done!");
}
