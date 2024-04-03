use crate::database;

use crate::plugins::PluginManager;
use crate::scraper;
use crate::scraper::InternalScraper;
use crate::sharedtypes;
use crate::sharedtypes::ScraperType;
use crate::threading;
use crate::time_func;
use ahash::AHashMap;

use log::info;

use rusqlite::Connection;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

pub struct Jobs {
    _jobid: Vec<u128>,
    _secs: usize,
    _sites: Vec<String>,
    _params: Vec<Vec<String>>,
    //References jobid in _inmemdb hashmap :D
    _jobstorun: Vec<usize>,
    _jobref: AHashMap<usize, (sharedtypes::DbJobsObj, scraper::InternalScraper)>,
    scrapermanager: scraper::ScraperManager,
}
/*#[derive(Debug, Clone)]
pub struct JobsRef {
    //pub _idindb: usize,       // What is my ID in the inmemdb
    pub _sites: String,       // Site that the user is querying
    pub _params: Vec<String>, // Anything that the user passes into the db.
    pub _jobsref: usize,      // reference time to when to run the job
    pub _jobstime: usize,     // reference time to when job is added
    pub _committype: CommitType,
    //pub _scraper: scraper::ScraperManager // Reference to the scraper that will be used
}*/

///
/// Jobs manager creates & manages jobs
///
impl Jobs {
    pub fn new(newmanager: scraper::ScraperManager) -> Self {
        Jobs {
            _jobid: Vec::new(),
            _sites: Vec::new(),
            _params: Vec::new(),
            _secs: 0,
            _jobstorun: Vec::new(),
            _jobref: AHashMap::new(),
            scrapermanager: newmanager,
        }
    }

    ///
    /// Loads jobs to run into _jobstorun
    ///
    pub fn jobs_get(&mut self, db: &database::Main) {
        self._secs = time_func::time_secs();
        let _ttl = db.jobs_get_max();
        let hashjobs = db.jobs_get_all();
        let beans = self.scrapermanager.scraper_get();
        for each in hashjobs {
            if time_func::time_secs() >= each.1.time.unwrap() + each.1.reptime.unwrap() {
                for eacha in beans {
                    let dbsite = each.1.site.to_owned();
                    if eacha._sites.contains(&dbsite.unwrap()) {
                        self._jobref
                            .insert(*each.0, (each.1.to_owned(), eacha.to_owned()));
                    }
                }
            }
        }
        let msg = format!(
            "Loaded {} jobs out of {} jobs. Didn't load {} Jobs due to lack of scrapers or timing.",
            &self._jobref.len(),
            db.jobs_get_max(),
            db.jobs_get_max() - self._jobref.len(),
        );
        info!("{}", msg);
        println!("{}", msg);
    }

    ///
    /// Runs jobs in a much more sane matter
    ///
    pub fn jobs_run_new(
        &mut self,
        adb: &mut Arc<Mutex<database::Main>>,
        thread: &mut threading::Threads,
        _alt_connection: &mut Connection,
        pluginmanager: &mut Arc<Mutex<PluginManager>>,
    ) {
        let dba = adb.clone();
        let mut db = dba.lock().unwrap();

        //let mut name_ratelimited: AHashMap<String, (u64, Duration)> = AHashMap::new();
        let mut scraper_and_job: AHashMap<InternalScraper, Vec<sharedtypes::DbJobsObj>> =
            AHashMap::new();
        //let mut job_plus_storeddata: AHashMap<String, String> = AHashMap::new();

        // Checks if their are no jobs to run.
        if self.scrapermanager.scraper_get().is_empty() || self._jobref.is_empty() {
            println!("No jobs to run...");
            return;
        } else {
            // Loads DB into memory. Everything that hasn't been loaded already
            db.load_table(&sharedtypes::LoadDBTable::All);
        }

        // Appends ratelimited into hashmap for multithread scraper.
        for scrape in self.scrapermanager.scraper_get() {
            let name_result = db.settings_get_name(&format!("{:?}_{}", scrape._type, scrape._name));
            let _info = String::new();

            // Handles loading of settings into DB.Either Manual or Automatic to describe the functionallity
            match name_result {
                Some(_) => {
                    //dbg!(name_result);
                    //&name_result.unwrap().name
                }
                None => {
                    let isolatedtitle = format!("{:?}_{}", scrape._type, scrape._name);

                    let (_cookie, cookie_name) = self.library_cookie_needed(scrape);

                    db.setting_add(
                        isolatedtitle,
                        Some("Automatic Scraper".to_owned()),
                        None,
                        Some(cookie_name),
                        true,
                    );
                }
            };
            // Loops through all jobs in the ref. Adds ref into
            for each in &self._jobref {
                let job = each.1;

                // Checks job type. If manual then scraper handles ALL calls from here on.
                // If Automatic then jobs will handle it.
                match job.1._type {
                    ScraperType::Manual => {}
                    ScraperType::Automatic => {
                        // Checks if InternalScraper types are the same data.
                        if &job.1 == scrape {
                            match scraper_and_job.entry(job.1.clone()) {
                                Entry::Vacant(e) => {
                                    e.insert(vec![job.0.clone()]);
                                }
                                Entry::Occupied(mut e) => {
                                    e.get_mut().push(job.0.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Loops through each InternalScraper and creates a thread for it.
        for each in scraper_and_job {
            let scraper = each.0;

            // Captures the libloading library from the _library.
            // Removes item from hashmap so the thread can have ownership of libloaded scraper.
            let scrap = self.scrapermanager._library.remove(&scraper).unwrap();
            let jobs = each.1;

            thread.startwork(scraper, jobs, adb, scrap, pluginmanager);
        }
    }
    ///
    /// pub fn cookie_needed(&mut self, id: usize, params: String) -> (bool, String)
    ///
    pub fn library_cookie_needed(&self, memid: &InternalScraper) -> (ScraperType, String) {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::cookie_need(libloading)
        //self.scrapermanager.cookie_needed(memid)
    }
    /*///
    /// Automatic job running.
    ///
    ///
    pub fn automatic_job_run(
        source_url: &String,
        ratelimiter_object: &mut ratelimit::Ratelimiter,
        client: &mut Client,
        commit_type: sharedtypes::CommitType,
    ) {
    }*/

    /*///
    /// Runs jobs as they are needed to.
    ///
    pub fn jobs_run(&mut self, db: &mut database::Main) {
        // Sets up and checks scrapers

        let loaded_params: AHashMap<u128, Vec<String>> = AHashMap::new();
        let mut loaded_params: AHashMap<u128, Vec<String>> = AHashMap::new();
        let mut ratelimit: AHashMap<u128, (u64, Duration)> = AHashMap::new();

        // Handles any thing if theirs nothing to load.
        dbg!(&self._params);
        if self.scrapermanager.scraper_get().is_empty() || self._params.is_empty() {
            println!("No jobs to run...");
            return;
        }

        for each in 0..self.scrapermanager.scraper_get().len() {
            let name = self.scrapermanager.scraper_get()[each].name_get();

            dbg!(&format!("manual_{}", name));

            let name_result = db.settings_get_name(&format!("manual_{}", name));
            let each_u128: u128 = each.try_into().unwrap();
            let mut to_load = Vec::new();
            match name_result {
                Ok(_) => {
                    println!("Dont have to add manual to db.");

                    let rlimit = self.scrapermanager.scraper_get()[each].ratelimit_get();
                    to_load.push(self._params[each][0].to_string());
                    to_load.push(name_result.unwrap().1.to_string());

                    loaded_params.insert(each_u128, to_load);
                    ratelimit.insert(each_u128, rlimit);
                }
                Err("None") => {
                    let rlimit = self.scrapermanager.scraper_get()[each].ratelimit_get();
                    let (cookie, cookie_name) = self.library_cookie_needed(
                        self._jobstorun[each].into(),
                        self._params[each][0].to_string(),
                    );
                    db.setting_add(
                        format!(
                            "manual_{}",
                            self.scrapermanager.scraper_get()[each].name_get()
                        ),
                        "Manually controlled scraper.".to_string(),
                        0,
                        cookie_name.to_string(),
                        true,
                    );
                    to_load.push(self._params[each][0].to_string());
                    loaded_params.insert(each_u128, to_load);
                    ratelimit.insert(each_u128, rlimit);
                }
                Err(&_) => continue,
            };
        }

        // setup for scraping jobs will probably outsource this to another file :D.
        for each in 0..self._jobstorun.len() {
            let each_u128: u128 = each.try_into().unwrap();
            println!(
                "Running Job: {} {} {:?}",
                self._jobstorun[each], self._sites[each], self._params[each]
            );

            let parzd: Vec<&str> = self._params[each][0].split(' ').collect::<Vec<&str>>();
            let mut parsed: Vec<String> = Vec::new();
            for a in parzd {
                parsed.push(a.to_string());
            }

            let index: usize = self._jobstorun[each].into();

            // url is the output from the designated scraper that has the correct

            let bools: Vec<bool> = Vec::new();

            let url: Vec<String> =
                self.library_url_dump(self._jobstorun[each].into(), &loaded_params[&each_u128]);

            let boo = self.library_download_get(self._jobstorun[each].into());
            //let mut ratelimiter = block_on(download::ratelimiter_create(ratelimit[&each_u128]));
            if boo {
                break;
            }
            let beans =
                download::dltext(url, &mut self.scrapermanager, self._jobstorun[each].into());
            println!("Downloading Site: {}", &each);
            // parses db input and adds tags to db.
            let (url_vec, urln_vec) = db.parse_input(&beans);
            let urls_to_remove: Vec<String> = Vec::new();

            // Filters out already downloaded files.
            let namespace_id = db.namespace_get(&"parsed_url".to_string()).0;
            let mut cnt = 0;

            let location = db.settings_get_name(&"FilesLoc".to_string()).unwrap().1;
            file::folder_make(&(location).to_string());

            // Total files that are already downloaded.
            // Re-adds tags & relationships into DB Only enable if their are changes to scrapers.
            dbg!(&loaded_params[&each_u128]);
            if self._params[each][1] == "true" {
                for urls in urln_vec.keys() {
                    dbg!(format!("Checking url for tags: {}", &urls));

                    let url_id = db.tag_get_name(urls.to_string(), namespace_id).0;
                    let fileids = db.relationship_get_fileid(&url_id);
                    for fids in &fileids {
                        for tags in &urln_vec[urls] {
                            db.tag_add(tags.0.to_string(), "".to_string(), tags.1, true);
                            let tagid = db.tag_get_name(tags.0.to_string(), tags.1).0;
                            db.relationship_add(*fids, tagid, true);
                        }
                    }
                }
            }

            let utl_total = url_vec.len();

            dbg!(format!("Total Files pulled: {}", &url_vec.len()));
            for urls in url_vec.keys() {
                let map = download::file_download(urls, &location);
                println!("Downloading file# : {} / {}", &cnt, &utl_total);

                // Populates the db with files.
                for every in map.0.keys() {
                    db.file_add(
                        0,
                        map.0[every].to_string(),
                        map.1.to_string(),
                        location.to_string(),
                        true,
                    );
                    cnt += 1;
                }

                // Populates the db with relations.
                let hash = db.file_get_hash(&map.0[&urls.to_string()]).0;
                let url_namespace = db.namespace_get(&"parsed_url".to_string()).0;
                db.tag_add(urls.to_string(), "".to_string(), url_namespace, true);
                let urlid = db.tag_get_name(urls.to_string(), url_namespace).0;
                db.relationship_add(hash, urlid, true);
                for tags in &url_vec[urls] {
                    db.tag_add(tags.0.to_string(), "".to_string(), tags.1, true);
                    let tagid = db.tag_get_name(tags.0.to_string(), tags.1).0;
                    db.relationship_add(hash, tagid, true);
                }
            }
        }
    }*/

    /*/// ALL of the lower functions are just wrappers for the scraper library.
    /// This is nice because their's no unsafe code anywhere else inside code base.

    ///
    /// Returns a url to grab for.
    ///
    pub fn library_url_get(
        &mut self,
        memid: &InternalScraper,
        params: &Vec<sharedtypes::ScraperParam>,
    ) -> Vec<String> {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::url_load(libloading, params)
        //self.scrapermanager.url_load(memid, params.to_vec())
    }

    ///
    /// Parses stuff from dltext.
    ///
    pub fn library_parser_call(
        &mut self,
        memid: &InternalScraper,
        params: &String,
    ) -> Result<sharedtypes::ScraperObject, sharedtypes::ScraperReturn> {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::parser_call(libloading, params)
        //self.scrapermanager.parser_call(memid, params)
    }

    ///
    /// Returns a url to grab for.
    ///
    pub fn library_url_dump(
        &self,
        memid: &InternalScraper,
        params: &Vec<sharedtypes::ScraperParam>,
    ) -> Vec<String> {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::url_dump(libloading, params)
        //self.scrapermanager.url_dump(memid, params.to_vec())
    }
    ///
    /// pub fn cookie_needed(&mut self, id: usize, params: String) -> (bool, String)
    ///
    pub fn library_cookie_needed(&self, memid: &InternalScraper) -> (ScraperType, String) {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::cookie_need(libloading)
        //self.scrapermanager.cookie_needed(memid)
    }

    ///
    /// Tells system if scraper should handle downloads.
    ///
    pub fn library_download_get(&self, memid: &InternalScraper) -> bool {
        let libloading = self.scrapermanager.returnlibloading(memid);
        scraper::scraper_download_get(libloading)
        //self.scrapermanager.scraper_download_get(memid)
    }*/
}
