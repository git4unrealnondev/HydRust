use crate::database;
use crate::download;

use crate::logging;
//use crate::jobs::JobsRef;
use crate::logging::info_log;
use crate::plugins::PluginManager;
use crate::scraper;

use crate::sharedtypes;
use crate::sharedtypes::JobScraper;
use crate::sharedtypes::ScraperReturn;

use async_std::task;
use futures;

//use log::{error, info};
use ratelimit::Ratelimiter;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct Threads {
    _workers: usize,
    worker: Vec<Worker>,
}

///
/// Holder for workers.
/// Workers manage their own threads.
///
impl Threads {
    pub fn new() -> Self {
        let workers = 0;
        let worker = Vec::new();

        Threads {
            _workers: workers,
            worker,
        }
    }

    ///
    /// Adds a worker to the threadvec.
    ///
    pub fn startwork(
        &mut self,
        scraper: scraper::InternalScraper,
        jobs: Vec<sharedtypes::DbJobsObj>,
        db: &mut Arc<Mutex<database::Main>>,
        scrapermanager: libloading::Library,
        pluginmanager: &mut Arc<Mutex<PluginManager>>,
    ) {
        let worker = Worker::new(
            self._workers,
            scraper,
            jobs,
            //&mut self._runtime,
            db,
            scrapermanager,
            pluginmanager,
        );
        self._workers += 1;
        self.worker.push(worker);

        //self._workers.push(worker);
    }
}
///
/// Worker holder for data. Will add a scraper processor soon tm.
///
struct Worker {
    id: usize,
    scraper: scraper::InternalScraper,
    jobs: Vec<sharedtypes::DbJobsObj>,
    thread: Option<std::thread::JoinHandle<()>>,
}

///
/// When code get deleted (cleaned up. This code runs.)
///  Cleans thread from pool.  
///
/*impl Drop for threads {
    fn drop(&mut self) {
        for worker in &mut self._workers {
            if let Some(thread) = worker.thread.take() {
                info!("Shutting Down Worker from ThreadManager: {}", worker.id);
                futures::executor::block_on(async { thread.await.unwrap()});
            }
        }
    }
}*/

///
/// closes the thread that the worker contains.
/// Used in easy thread handeling
/// Only reason to do this over doing this with default drop behaviour is the logging.
///
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            info_log(&format!("Shutting Down Worker from Worker: {}", self.id));
            println!("Shutting Down Worker from Worker: {}", self.id);
            futures::executor::block_on(async { thread.join().unwrap() });
        }
    }
}

impl Worker {
    fn new(
        id: usize,
        scraper: scraper::InternalScraper,
        jobs: Vec<sharedtypes::DbJobsObj>,
        //rt: &mut Runtime,
        dba: &mut Arc<Mutex<database::Main>>,
        libloading: libloading::Library,
        pluginmanager: &mut Arc<Mutex<PluginManager>>,
    ) -> Worker {
        info_log(&format!(
            "Creating Worker for id: {} Scraper Name: {} With a jobs length of: {}",
            &id,
            &scraper._name,
            &jobs.len()
        ));
        let mut db = dba.clone();
        let mut jblist = jobs.clone();
        let manageeplugin = pluginmanager.clone();
        let scrap = scraper.clone();
        let thread = thread::spawn(move || {
            let mut job_params: HashSet<JobScraper> = HashSet::new();
            let mut job_ref_hash: HashMap<JobScraper, sharedtypes::DbJobsObj> = HashMap::new();
            let mut rate_limit_vec: Vec<Ratelimiter> = Vec::new();
            let mut rate_limit_key: HashMap<String, usize> = HashMap::new();

            // Main loop for processing
            // All queries have been deduplicated.
            let mut job_loop = true;
            while job_loop {
                for job in jblist.clone() {
                    let mut par_vec: Vec<sharedtypes::ScraperParam> = Vec::new();
                    let parpms: Vec<String> = job
                        .param
                        .as_ref()
                        .unwrap()
                        .split_whitespace()
                        .map(str::to_string)
                        .collect();
                    for par in parpms {
                        let temp = sharedtypes::ScraperParam {
                            param_data: par,
                            param_type: sharedtypes::ScraperParamType::Normal,
                        };
                        par_vec.push(temp)
                    }

                    {
                        // Gets info from DB. If it exists then insert into params hashset.
                        let unwrappydb = &mut db.lock().unwrap();
                        let datafromdb = unwrappydb
                            .settings_get_name(&format!(
                                "{}_{}",
                                scrap._type,
                                scrap._name.to_owned()
                            ))
                            .unwrap()
                            .param
                            .clone();
                        match datafromdb {
                            None => {}
                            Some(param) => {
                                // Adds database tag if applicable.
                                let scrap_data = sharedtypes::ScraperParam {
                                    param_data: param,
                                    param_type: sharedtypes::ScraperParamType::Database,
                                };

                                par_vec.push(scrap_data);
                            }
                        }
                    }

                    let sc = JobScraper {
                        site: job.site.clone().unwrap(),
                        param: par_vec,
                        original_param: job.param.clone().unwrap(),
                        job_type: job.jobtype,
                        //job_ref: job.clone(),
                    };
                    job_ref_hash.insert(sc.clone(), job);
                    job_params.insert(sc);
                }
                // dbg!(&job_params);

                for job in &job_params.clone() {
                    let urlload = match job.job_type {
                        sharedtypes::DbJobType::Params => {
                            scraper::url_dump(&libloading, &job.param)
                        }
                        sharedtypes::DbJobType::Plugin => {
                            continue;
                        }
                        sharedtypes::DbJobType::FileUrl => {
                            let parpms: Vec<String> = job
                                .original_param
                                .split_whitespace()
                                .map(str::to_string)
                                .collect();
                            parpms
                        }
                        sharedtypes::DbJobType::Scraper => {
                            vec![job.original_param.clone()]
                        }
                    };
                    // Only instants the ratelimit if we don't already have it.
                    let mut ratelimit = match rate_limit_key.get_mut(&job.site) {
                        None => {
                            info_log(&format!("Creating ratelimiter for site: {}", &job.site));
                            let u_temp = rate_limit_vec.len();
                            rate_limit_key.insert(job.site.clone(), u_temp);
                            rate_limit_vec.push(download::ratelimiter_create(
                                scrap._ratelimit.0,
                                scrap._ratelimit.1,
                            ));
                            &mut rate_limit_vec[u_temp]
                        }
                        Some(u_temp) => rate_limit_vec.get_mut(*u_temp).unwrap(),
                    };

                    let mut client = download::client_create();

                    'urlloop: for urll in urlload {
                        'errloop: loop {
                            download::ratelimiter_wait(&mut ratelimit);
                            let resp = task::block_on(download::dltext_new(
                                urll.to_string(),
                                &mut ratelimit,
                                &mut client,
                            ));
                            let st = match resp {
                                Ok(respstring) => scraper::parser_call(&libloading, &respstring),
                                Err(_) => continue,
                            };

                            let out_st = match st {
                                Ok(objectscraper) => objectscraper,
                                Err(ScraperReturn::Nothing) => {
                                    job_params.remove(job);
                                    dbg!("Exiting loop due to nothing.");
                                    break 'urlloop;
                                }
                                Err(ScraperReturn::EMCStop(emc)) => {
                                    panic!("EMC STOP DUE TO: {}", emc);
                                }
                                Err(ScraperReturn::Stop(stop)) => {
                                    logging::error_log(&format!(
                                        "Stopping job: {:?} due to {}",
                                        job.param, stop
                                    ));
                                    job_params.remove(job);
                                    continue;
                                }
                                Err(ScraperReturn::Timeout(time)) => {
                                    let time_dur = Duration::from_secs(time);
                                    thread::sleep(time_dur);
                                    continue;
                                }
                            };
                            //Parses tags from urls
                            for tag in out_st.tag {
                                let to_parse = parse_tags(&mut db, tag, None);
                                for each in to_parse {
                                    job_params.insert(each);
                                }
                            }
                            // Parses files from urls
                            for file in out_st.file {
                                if let Some(ref source) = file.source_url {
                                    {
                                        let mut source_url_id = {
                                            let unwrappydb = &mut db.lock().unwrap();
                                            unwrappydb
                                                .namespace_get(&"source_url".to_string())
                                                .cloned() // defaults to 0 due to unknown.
                                        };

                                        if source_url_id.is_none() {
                                            // Namespace doesn't exist. Will create
                                            let unwrappydb = &mut db.lock().unwrap();
                                            unwrappydb.namespace_add(
                                                "source_url".to_string(),
                                                Some("Source URL for a file.".to_string()),
                                                true,
                                            );
                                            source_url_id = unwrappydb
                                                .namespace_get(&"source_url".to_string())
                                                .cloned();
                                        }

                                        // If url exists in db then don't download
                                        let url_tag = {
                                            let unwrappydb = db.lock().unwrap();
                                            unwrappydb
                                                .tag_get_name(
                                                    source.clone(),
                                                    source_url_id.unwrap().clone(),
                                                )
                                                .cloned()
                                        };

                                        let location = {
                                            let unwrappydb = &mut db.lock().unwrap();
                                            unwrappydb.location_get()
                                        };

                                        // Get's the hash & file ext for the file.
                                        let fileid = match url_tag {
                                            None => {
                                                // Download file doesn't exist.
                                                download::ratelimiter_wait(&mut ratelimit);
                                                // URL doesn't exist in DB Will download
                                                info_log(&format!(
                                                    "Downloading: {} to: {}",
                                                    &source, &location
                                                ));
                                                let blopt;
                                                {
                                                    blopt = task::block_on(download::dlfile_new(
                                                        &client,
                                                        &file,
                                                        &location,
                                                        &mut Some(manageeplugin.clone()),
                                                    ));
                                                }
                                                let (hash, file_ext) = match blopt {
                                                    None => {
                                                        continue;
                                                    }
                                                    Some(blo) => blo,
                                                };
                                                let fileid;
                                                {
                                                    let unwrappydb = &mut db.lock().unwrap();
                                                    fileid = unwrappydb.file_add(
                                                        None, &hash, &file_ext, &location, true,
                                                    );
                                                    let tagid = unwrappydb.tag_add(
                                                        source.to_string(),
                                                        source_url_id.unwrap().clone(),
                                                        true,
                                                        None,
                                                    );
                                                    unwrappydb
                                                        .relationship_add(fileid, tagid, true);
                                                }
                                                fileid
                                            }
                                            Some(url_id) => {
                                                let file_id;
                                                {
                                                    // We've already got a valid relationship
                                                    let unwrappydb = &mut db.lock().unwrap();
                                                    file_id = unwrappydb
                                                        .relationship_get_one_fileid(&url_id)
                                                        .copied();
                                                    if let Some(fid) = file_id {
                                                        unwrappydb.file_get_id(&fid).unwrap();
                                                    }
                                                }
                                                // fixes busted links.
                                                if let Some(file_id) = file_id {
                                                    info_log(&format!(
                                                    "Skipping file: {} Due to already existing in Tags Table.",
                                                    &source
                                                ));

                                                    file_id.clone()
                                                } else {
                                                    // Fixes the link between file and url
                                                    // tag.

                                                    download::ratelimiter_wait(&mut ratelimit);
                                                    // URL doesn't exist in DB Will download
                                                    info_log(&format!(
                                                        "Downloading: {} to: {}",
                                                        &source, &location
                                                    ));
                                                    let blopt;
                                                    {
                                                        blopt =
                                                            task::block_on(download::dlfile_new(
                                                                &client,
                                                                &file,
                                                                &location,
                                                                &mut Some(manageeplugin.clone()),
                                                            ));
                                                    }
                                                    let (hash, file_ext) = match blopt {
                                                        None => {
                                                            continue;
                                                        }
                                                        Some(blo) => blo,
                                                    };
                                                    let fileid;
                                                    {
                                                        let unwrappydb = &mut db.lock().unwrap();
                                                        fileid = unwrappydb.file_add(
                                                            None, &hash, &file_ext, &location, true,
                                                        );
                                                        let tagid = unwrappydb.tag_add(
                                                            source.to_string(),
                                                            source_url_id.unwrap().clone(),
                                                            true,
                                                            None,
                                                        );
                                                        unwrappydb
                                                            .relationship_add(fileid, tagid, true);
                                                    }
                                                    fileid
                                                }
                                            }
                                        };

                                        // We've got valid fileid for reference.
                                        for taz in file.tag_list {
                                            //dbg!(&taz);
                                            let tag = taz;

                                            let urls_scrap = parse_tags(&mut db, tag, Some(fileid));
                                            for urlz in urls_scrap {
                                                //let url_job = JobScraper {};
                                                //dbg!(&urlz);
                                                job_params.insert(urlz);
                                                //      job_ref_hash.insert(urlz, job);
                                            }
                                        }
                                    }
                                }
                                // End of err catching loop.
                                // break 'errloop;
                            }
                            break 'errloop;
                        }

                        //dbg!("End of URL Loop");
                        //let unwrappydb = &mut db.lock().unwrap();
                        //unwrappydb.transaction_flush();
                    }
                    //println!("End of loop");
                    let unwrappydb = &mut db.lock().unwrap();
                    //dbg!(&job);
                    unwrappydb.del_from_jobs_table(&"param".to_owned(), &job.original_param);
                    job_params.remove(&job);
                    logging::info_log(&format!("Removing job {:?}", &job));

                    if let Some(jobscr) = job_ref_hash.get(job) {
                        let index = jblist.iter().position(|r| r == jobscr).unwrap();
                        jblist.remove(index);
                    }

                    unwrappydb.transaction_flush();
                }

                if job_params.is_empty() {
                    job_loop = false;
                }
            }
        });

        return Worker {
            id,
            thread: Some(thread),
            scraper,
            jobs,
        };
    }
}

///
/// Parses tags and adds the tags into the db.
///
fn parse_tags(
    db: &mut Arc<Mutex<database::Main>>,
    tag: sharedtypes::TagObject,
    file_id: Option<usize>,
) -> HashSet<sharedtypes::JobScraper> {
    let mut url_return: HashSet<sharedtypes::JobScraper> = HashSet::new();

    let unwrappy = &mut db.lock().unwrap();

    //dbg!(&tag);

    match tag.tag_type {
        sharedtypes::TagType::Normal => {
            //println!("Adding tag: {} {:?}", tag.tag, &file_id);
            // We've recieved a normal tag. Will parse.

            let namespace_id =
                unwrappy.namespace_add(tag.namespace.name, tag.namespace.description, true);
            let tag_id = unwrappy.tag_add(tag.tag, namespace_id, true, None);
            match tag.relates_to {
                None => {
                    /*let relate_ns_id = unwrappy.namespace_add(
                        relate.namespace.name.clone(),
                        relate.namespace.description,
                        true,
                    );*/
                }
                Some(relate) => {
                    let relate_ns_id = unwrappy.namespace_add(
                        relate.namespace.name.clone(),
                        relate.namespace.description,
                        true,
                    );
                    let relate_id = unwrappy.tag_add(relate.tag, relate_ns_id, true, None);
                    unwrappy.parents_add(namespace_id, tag_id, relate_ns_id, relate_id, true);
                }
            }

            match file_id {
                None => {}
                Some(id) => {
                    unwrappy.relationship_add(id, tag_id, true);
                }
            }
            url_return
        }
        sharedtypes::TagType::ParseUrl((jobscraped, skippy)) => {
            match skippy {
                sharedtypes::SkipIf::None => {
                    url_return.insert(jobscraped);
                }
                sharedtypes::SkipIf::Tag(taginfo) => 'tag: {
                    let nid = unwrappy.namespace_get(&taginfo.namespace.name);
                    let id = match nid {
                        None => {
                            println!("Namespace does not exist: {:?}", taginfo.namespace);
                            url_return.insert(jobscraped);
                            break 'tag;
                        }
                        Some(id) => id,
                    };

                    match unwrappy.tag_get_name(taginfo.tag.clone(), *id) {
                        None => {
                            println!("WillDownload: {}", taginfo.tag);
                            url_return.insert(jobscraped);
                        }
                        Some(tag_id) => {
                            if taginfo.needsrelationship {
                                let rel_hashset = unwrappy.relationship_get_fileid(tag_id);
                                match rel_hashset {
                                    None => {
                                        println!(
                                            "Downloading: {} because no relationship",
                                            taginfo.tag
                                        );
                                        println!("Will download from: {}", taginfo.tag);
                                        url_return.insert(jobscraped);
                                        break 'tag;
                                    }
                                    Some(_) => {
                                        println!(
                                            "Skipping because this already has a relationship. {}",
                                            taginfo.tag
                                        );

                                        //println!("Will download from: {}", taginfo.tag);
                                        //url_return.insert(jobscraped);
                                        break 'tag;
                                    }
                                }
                            }
                            println!("Ignoring: {}", taginfo.tag);

                            break 'tag;
                        }
                    }
                }
            }
            // Returns the url that we need to parse.
            url_return
        }
        sharedtypes::TagType::Special => {
            // Do nothing will handle this later lol.
            url_return
        }
    }
}
