#![allow(dead_code)]
#![allow(unused_variables)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString};

///
/// Database Tags Object
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Clone)]
pub struct DbTagNNS {
    pub name: String,
    pub namespace: usize,
}
///
/// Database Tags Object
///
#[derive(Debug, Deserialize, Serialize)]
pub struct DbTagObjCompatability {
    pub id: usize,
    pub name: String,
    pub parents: Option<usize>,
    pub namespace: usize,
}

#[derive(Debug, EnumIter, Display)]
// Manages what the search can do.
pub enum Search {
    Fid(Vec<String>),
    Tid(Vec<String>),
    Tag(Vec<String>),
    Hash(Vec<String>),
}
#[derive(Debug, EnumIter, Display)]
pub enum Tasks {
    Csv(String, CsvCopyMvHard), // CSV importation. cp mv hardlink
    Remove(TasksRemove),
}

#[derive(Debug, EnumIter, Display)]
pub enum TasksRemove {
    RemoveNamespaceId(usize),
    RemoveNamespaceString(String),
    None,
}
impl Default for TasksRemove {
    fn default() -> Self {
        TasksRemove::None
    }
}
///
/// Determines how to run a function
///
#[derive(Debug, EnumIter, Display)]
pub enum PluginThreadType {
    Inline,        // Run plugin inside of the calling function. DEFAULT
    Spawn,         // Spawns a new thread, Runs concurrently to the calling function.
    SpawnBlocking, // Spawns a new thread, Blocks the main thread until the plugin finishes work.
    Daemon, // Spawns a thread as a daemon all calls to the plugin will be routed to the daemon thread.
}

///
/// Information about CSV what we should do with the files.
///
#[derive(Debug, EnumIter, Display, Default)]
pub enum CsvCopyMvHard {
    #[default]
    Copy,
    Move,
    Hardlink,
}

///
/// Tells DB which table to load.
///
#[derive(EnumIter, PartialEq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum LoadDBTable {
    Files,        // Files table
    Jobs,         // Jobs table
    Namespace,    // Namespace mapping
    Parents,      // Parents mapping table
    Relationship, // Relationships table
    Settings,     // Settings table
    Tags,         // Tags storage table
    All,          // Loads all unloaded tables.
}

#[allow(dead_code)]
pub fn stringto_commit_type(into: &String) -> CommitType {
    for each in CommitType::iter() {
        if into == &each.to_string() {
            return each;
        }
    }
    let mut panic = "Could Not format CommitType as one of: ".to_string();
    for each in CommitType::iter() {
        panic += format!("{} ", each).as_str();
    }

    panic!("{}", panic);
}

///
/// Dummy Holder
/// Dummy thick
///
#[allow(dead_code)]
#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum SearchHolder {
    AND((usize, usize)),
    OR((usize, usize)),
    NOT((usize, usize)),
}

///
/// Allows searching inside the db.
/// search_relate relates the item in the vec with eachother
/// the IDs in search relate correspond to the id's in searches.
/// IE: if 0 & 1 are an AND search then the 4 search items are AND searched in db in addition to
/// the 4 terms in the searches
///
#[allow(dead_code)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchObj {
    pub search_relate: Option<Vec<SearchHolder>>,
    pub searches: Vec<SearchHolder>,
}

#[allow(dead_code)]
pub fn stringto_search_type(into: &String) -> Search {
    for each in Search::iter() {
        if into == &each.to_string() {
            return each;
        }
    }
    let mut panic = "Could Not format CommitType as one of: ".to_string();
    for each in Search::iter() {
        panic += format!("{} ", each).as_str();
    }

    panic!("{}", panic);
}

#[allow(dead_code)]
pub fn stringto_jobtype(into: &String) -> DbJobType {
    for each in DbJobType::iter() {
        if into == &each.to_string() {
            return each;
        }
    }
    let mut panic = "Could not format DbJobType as one of:".to_string();
    for each in DbJobType::iter() {
        panic += format!("{} ", each).as_str();
    }
    panic!("{}", panic);
}
#[derive(Debug, EnumIter, Clone, Eq, Hash, PartialEq, Copy, EnumString)]
pub enum CommitType {
    StopOnNothing, // Processes all files and data doesn't stop processing.
    StopOnFile,    // Stops processing if it sees a file it's already seen.
                   //SkipOnFile,
                   //AddToDB,
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, EnumIter, Clone, PartialEq, Hash, Eq, Display)]
pub enum ScraperType {
    Manual,
    Automatic,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ScraperReturn {
    EMCStop(String), // STOP IMMEDIENTLY: ISSUE WITH SITE : PANICS no save
    Nothing,         // Hit nothing to search. Move to next job.
    Stop(String),    // Stop current job, Record issue Move to next.
    Timeout(u64),    // Wait X seconds before retrying.
}

///
/// What the scraper passes between loaded 3rd party scrapers and the internal scrpaer.
///
#[derive(Debug)]
pub struct ScraperObject {
    pub file: HashSet<FileObject>,
    pub tag: HashSet<TagObject>,
}

///
/// Defines what we need to reimport a file to derive a source URL.
/// Currently only support hash.
///
pub struct ScraperFileRegen {
    pub hash: HashesSupported,
}

///
/// Input for the scraper to parse the info from the system.
///
pub struct ScraperFileInput {
    pub hash: Option<String>,
    pub ext: Option<String>,
}

///
/// Defines what data the scraper will return.
/// Will likely be a source URL or if we can't parse from hash.
///
#[derive(Debug)]
pub struct ScraperFileReturn {
    pub tag: Option<GenericNamespaceObj>,
}

///
/// File object
/// should of done this sooner lol
///
#[derive(Debug, Deserialize, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct DbFileObj {
    pub id: Option<usize>,
    pub hash: String,
    pub ext: String,
    pub location: String,
}

///
/// File object
/// Should only be used for parsing data from plugins
///
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginFileObj {
    pub id: Option<usize>,
    pub hash: Option<String>,
    pub ext: Option<String>,
    pub location: Option<String>,
}

///
/// Namespace object
/// should of done this sooner lol
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbNamespaceObj {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
}

///
/// Namespace object
/// should of done this sooner lol
///
/*#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct PluginRelatesObj {
    pub tag: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}*/

///
/// Database Jobs object
///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct DbJobsObj {
    pub time: Option<usize>,
    pub reptime: Option<usize>,
    pub site: Option<String>,
    pub param: Option<String>,
    pub jobtype: DbJobType,
    pub committype: Option<CommitType>,
}

///
/// Type of job in db.
/// Will be used to confirm what the scraping logic should work.
///
#[derive(Debug, Copy, Hash, Eq, PartialEq, Clone, EnumIter, Display)]
pub enum DbJobType {
    Params,  // Default recognises this as a param query.
    Plugin,  // Runs a plugin with the data inside it.
    FileUrl, // This is a url containing files
    Scraper, // Sends this to a scraper directly.
}

///
/// Database Parents Object.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DbParentsObj {
    pub tag_namespace_id: usize,
    pub tag_id: usize,
    pub relate_namespace_id: usize,
    pub relate_tag_id: usize,
}

///
/// Database Relationship Object
///
#[derive(Debug)]
pub struct DbRelationshipObj {
    pub fileid: usize,
    pub tagid: usize,
}

///
/// Database Settings Object
///
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DbSettingObj {
    pub name: String,
    pub pretty: Option<String>,
    pub num: Option<usize>,
    pub param: Option<String>,
}

///
/// Database search object
///
#[derive(Debug)]
pub struct DbSearchObject {
    pub tag: String,
    pub namespace: Option<String>,
    pub namespace_id: Option<usize>,
}

///
/// Database search enum between 2 tags
///
#[derive(Debug)]
pub enum DbSearchTypeEnum {
    AND,
    OR,
}

///
/// Database search query
///
#[derive(Debug)]
pub struct DbSearchQuery {
    pub tag_one: DbSearchObject,
    pub tag_two: DbSearchObject,
    pub search_enum: DbSearchTypeEnum,
}

///
/// Database Relationship For Plugin passing
///
#[derive(Debug)]
pub struct DbPluginRelationshipObj {
    pub file_hash: String,
    pub tag_name: String,
    pub tag_namespace: String,
}

#[derive(Debug)]
pub enum DBPluginOutputEnum {
    Add(Vec<DBPluginOutput>),
    Del(Vec<DBPluginOutput>),
    None,
}

#[derive(PartialEq, Debug)]
pub struct DBPluginTagOut {
    pub name: String,
    pub parents: Option<Vec<DbPluginParentsObj>>,
    pub namespace: String,
}

///
/// Database Parents Object.
///
#[derive(Debug, PartialEq)]
pub struct DbPluginParentsObj {
    pub tag_namespace_string: String,
    pub relate_namespace_id: String,
    pub relate_tag_id: String,
}

///
/// Namespace for plugin objects
///
#[derive(Debug, PartialEq)]
pub struct DbPluginNamespace {
    pub name: String,
    pub description: Option<String>,
}

///
/// Plugin output for the passed object
///
#[derive(Debug)]
pub struct DBPluginOutput {
    pub tag: Option<Vec<DBPluginTagOut>>,   // Adds a tag to DB
    pub setting: Option<Vec<DbSettingObj>>, // Adds a setting
    pub relationship: Option<Vec<DbPluginRelationshipObj>>, // Adds a relationship into the DB.
    pub parents: Option<Vec<DbParentsObj>>, // Adds a parent object in db
    pub jobs: Option<Vec<DbJobsObj>>,       // Adds a job
    pub namespace: Option<Vec<DbPluginNamespace>>, // Adds a namespace
    pub file: Option<Vec<PluginFileObj>>,   // Adds a file into db
}

///
/// Represents one file
///
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct FileObject {
    pub source_url: Option<String>,
    pub hash: Option<HashesSupported>, // Hash of file
    pub tag_list: Vec<TagObject>,
}

///
/// Holder of Tag info.
/// Keeps relationalship info into account.
///
#[derive(Debug, Eq, PartialEq, Hash)]

pub struct TagObject {
    pub namespace: GenericNamespaceObj,
    pub tag: String,
    pub tag_type: TagType,
    pub relates_to: Option<SubTag>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SubTag {
    pub namespace: GenericNamespaceObj,
    pub tag: String,
    pub tag_type: TagType,
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct GenericNamespaceObj {
    pub name: String,
    pub description: Option<String>,
}

///
/// Tag Type object. Represents metadata for parser.
///
#[derive(Debug, Clone)]
#[allow(dead_code)]
#[derive(Eq, Hash, PartialEq)]
pub enum TagType {
    Normal,                         // Normal tag.
    ParseUrl((JobScraper, SkipIf)), // Scraper to download and parse a new url.
    Special, // Probably will add support for something like file descriptors or plugin specific things.
             //
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub tag: String,
    pub namespace: GenericNamespaceObj,
    pub needsrelationship: bool, // If theirs a relationship then we will not add it to the checkers.
}

///
/// Used for skipping a ParseUrl in TagType if a tag exists.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkipIf {
    Tag(Tag),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobScraper {
    pub site: String,
    pub param: Vec<ScraperParam>,
    pub original_param: String,
    pub job_type: DbJobType,
    //pub job_ref: DbJobsObj,
}
///
/// Supported types of hashes in Rust Hydrus
///
#[derive(Debug, Clone, Display)]
#[allow(dead_code)]
#[derive(Eq, Hash, PartialEq)]
pub enum HashesSupported {
    Md5(String),
    Sha1(String),
    Sha256(String),
    None,
}

#[derive(Debug)]
pub struct JobsAdd {
    pub site: String,
    pub query: String,
    pub time: String,
    pub committype: CommitType,
}

#[derive(Debug)] // Manages what the jobs are.
pub struct JobsRemove {
    pub site: String,
    pub query: String,
    pub time: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AllFields {
    JobsAdd(JobsAdd),
    JobsRemove(JobsRemove),
    Search(Search),
    Nothing,
    Tasks(Tasks),
}

///
/// Plugin Callable actions for callbacks
///
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PluginCallback {
    OnDownload, // Ran when a file is downloaded
    OnStart,    // Starts when the software start
}

///
/// information block for plugin info
///
#[derive(Debug)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub version: f32,
    pub api_version: f32,
    pub callbacks: Vec<PluginCallback>,
    pub communication: Option<PluginSharedData>,
}

///
///
///
#[derive(Debug)]
pub struct PluginSharedData {
    pub thread: PluginThreadType,
    pub com_channel: Option<PluginCommunicationChannel>,
}

///
///
///
#[derive(Debug)]
pub enum PluginCommunicationChannel {
    Pipe(String),
    None,
}

// let temp = AllFields::JobsAdd(JobsAdd{Site: "yeet".to_owned(), Query: "yeet".to_owned(), Time: "Lo".to_owned(), Loop: "yes".to_owned(), ReCommit: "Test".

///
/// Straper type passed to params
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScraperParamType {
    Normal,
    Database,
}

///
/// Used to hold Scraper Parameters in db.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScraperParam {
    pub param_data: String,
    pub param_type: ScraperParamType,
}
