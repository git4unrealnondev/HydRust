use super::plugins::PluginManager;
//extern crate urlparse;
use super::sharedtypes;
use bytes::Bytes;
use file_format::FileFormat;
use log::{error, info};
use md5;
use reqwest::Client;
use sha1;
use sha2::Digest as sha2Digest;
use sha2::Sha512;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;
use url::Url;
extern crate reqwest;
use crate::helpers;
use async_std::task;
use ratelimit::Ratelimiter;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;

///
/// Makes ratelimiter and example
///
pub fn ratelimiter_create(number: u64, duration: Duration) -> Ratelimiter {
    println!(
        "Making ratelimiter with: {} Request Per: {:?}",
        &number, &duration
    );

    // The wrapper that implements ratelimiting
    Ratelimiter::builder(number, duration)
        .max_tokens(number)
        .initial_available(number)
        .build()
        .unwrap()
}

pub fn ratelimiter_wait(ratelimit_object: &mut Ratelimiter) {
    let limit = ratelimit_object.try_wait();
    match limit {
        Ok(_) => {}
        Err(sleep) => {
            std::thread::sleep(sleep);
        }
    }
}

///
/// Creates Client that the downloader will use.
///
///
pub fn client_create() -> Client {
    let useragent = "RustHydrusV1".to_string();
    // The client that does the downloading
    reqwest::ClientBuilder::new()
        .user_agent(useragent)
        .cookie_store(true)
        //.brotli(true)
        //.deflate(true)
        .gzip(true)
        .build()
        .unwrap()
}

///
/// Downloads text into db as responses. Filters responses by default limit if their's anything wrong with request.
///
pub async fn dltext_new(
    url_string: String,
    ratelimit_object: &mut Ratelimiter,
    client: &mut Client,
) -> Result<String, reqwest::Error> {
    //let mut ret: Vec<AHashMap<String, AHashMap<String, Vec<String>>>> = Vec::new();
    //let ex = Executor::new();

    let url = Url::parse(&url_string).unwrap();
    //let url = Url::parse("http://www.google.com").unwrap();

    //let requestit = Request::new(Method::GET, url);
    //fut.push();
    println!("Spawned web reach to: {}", &url_string);
    //let futureresult = futures::executor::block_on(ratelimit_object.ready())
    //    .unwrap()
    ratelimiter_wait(ratelimit_object);
    let futureresult = client.get(url).send().await;

    //let test = reqwest::get(url).await.unwrap().text();

    //let futurez = futures::executor::block_on(futureresult);
    //dbg!(&futureresult);

    match futureresult {
        Ok(_) => Ok(task::block_on(futureresult.unwrap().text()).unwrap()),
        Err(_) => Err(futureresult.err().unwrap()),
    }
}

///
/// Hashes the bytes and compares it to what the scraper should of recieved.
///
pub fn hash_bytes(bytes: &Bytes, hash: &sharedtypes::HashesSupported) -> (String, bool) {
    match hash {
        sharedtypes::HashesSupported::Md5(hash) => {
            let digest = md5::compute(bytes);
            //let sharedtypes::HashesSupported(hashe, _) => hash;
            (format!("{:x}", digest), &format!("{:x}", digest) == hash)
        }
        sharedtypes::HashesSupported::Sha1(hash) => {
            let mut hasher = sha1::Sha1::new();
            hasher.update(bytes);
            let hastring = format!("{:X}", hasher.finalize());
            let dune = &hastring == hash;
            (hastring, dune)
        }
        sharedtypes::HashesSupported::Sha256(hash) => {
            let mut hasher = Sha512::new();
            hasher.update(bytes);
            let hastring = format!("{:X}", hasher.finalize());
            let dune = &hastring == hash;
            (hastring, dune)
        }
        sharedtypes::HashesSupported::None => ("".to_string(), false),
    }
}

///
/// Downloads file to position
///
pub async fn dlfile_new(
    client: &Client,
    file: &sharedtypes::FileObject,
    location: &String,
    pluginmanager: &mut Option<Arc<Mutex<PluginManager>>>,
) -> Option<(String, String)> {
    let mut boolloop = true;
    let mut hash = String::new();
    let mut bytes: bytes::Bytes = Bytes::from(&b""[..]);
    let mut cnt = 0;
    while boolloop {
        let mut hasher = Sha512::new();

        let errloop = true;

        while errloop {
            let fileurlmatch = match &file.source_url {
                None => {
                    panic!(
                        "Tried to call dlfilenew when their was no file :C info: {:?}",
                        file
                    );
                }
                Some(fileurl) => fileurl,
            };

            let url = Url::parse(fileurlmatch).unwrap();

            let mut futureresult = client.get(url.as_ref()).send().await;
            loop {
                match futureresult {
                    Ok(_) => {
                        break;
                    }
                    Err(_) => {
                        error!("Repeating: {}", &url);
                        dbg!("Repeating: {}", &url);
                        let time_dur = Duration::from_secs(10);
                        thread::sleep(time_dur);
                        futureresult = client.get(url.as_ref()).send().await;
                    }
                }
            }

            // Downloads file into byte memory buffer
            let byte = futureresult.unwrap().bytes().await;

            // Error handling for dling a file.
            // Waits 10 secs to retry
            match byte {
                Ok(_) => {
                    bytes = byte.unwrap();
                    break;
                }
                Err(_) => {
                    error!("Repeating: {} , Due to: {:?}", &url, &byte.as_ref().err());
                    dbg!("Repeating: {} , Due to: {:?}", &url, &byte.as_ref().err());
                    let time_dur = Duration::from_secs(10);
                    thread::sleep(time_dur);
                }
            }
            if cnt >= 3 {
                return None;
            }
            cnt += 1;
        }

        hasher.update(&bytes.as_ref());

        // Final Hash
        hash = format!("{:X}", hasher.finalize());

        let parsedhash = match &file.hash {
            None => {
                panic!("DlFileNew: Cannot parse hash info : {:?}", &file);
            }
            Some(inputhash) => inputhash,
        };

        // Check and compare  to what the scraper wants
        let status = hash_bytes(&bytes, &parsedhash);

        // Logging
        if !status.1 {
            error!(
                "Parser file: {} FAILED HASHCHECK: {} {}",
                &parsedhash, status.0, status.1
            );
            cnt += 1;
        } else {
            info!("Parser returned: {} Got: {}", &parsedhash, status.0);
            //dbg!("Parser returned: {} Got: {}", &file.hash, status.0);
        }
        if cnt >= 3 {
            return None;
        }
        boolloop = !status.1;
    }

    let final_loc = helpers::getfinpath(&location, &hash);

    // Gives file extension
    let file_ext = FileFormat::from_bytes(&bytes).extension().to_string();

    let mut content = Cursor::new(bytes.clone());

    // Gets final path of file.
    let orig_path = format!("{}/{}", &final_loc, &hash);
    let mut file_path = std::fs::File::create(&orig_path).unwrap();

    // Copies file from memory to disk
    std::io::copy(&mut content, &mut file_path).unwrap();

    {
        // If the plugin manager is None then don't do anything plugin wise.
        // Useful for if doing something that we CANNOT allow plugins to run.
        if let Some(pluginmanager) = pluginmanager {
            pluginmanager
                .lock()
                .unwrap()
                .plugin_on_download(bytes.as_ref(), &hash, &file_ext);
        }
        // Wouldove rather passed teh Cursor or Bytes Obj but it wouldn't work for some reason with the ABI
    }
    println!("Downloaded hash: {}", &hash);

    Some((hash, file_ext))
}

///
/// Hashes file from location string with specified hash into the hash of the file.
///
pub fn hash_file(filename: &String, hash: &sharedtypes::HashesSupported) -> (String, Bytes) {
    let f = File::open(filename).unwrap();
    let mut reader = BufReader::new(f);
    let mut buf = Vec::new();

    reader.read_to_end(&mut buf).unwrap();
    let b = Bytes::from(buf);
    let hash_self = hash_bytes(&b, hash);

    (hash_self.0, b)
}
