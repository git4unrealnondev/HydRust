use clap::{arg, Parser, Subcommand};

///
/// From: git4unrealnondev
/// Das code sucks.
///
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct MainWrapper {
    #[command(subcommand)]
    pub a: Option<test>,
}

#[derive(Debug, Parser)]
pub enum test {
    /// Manages their jobs in db.
    #[clap(subcommand)]
    Job(JobStruct),
    /// Searches the DB.
    #[clap(subcommand)]
    Search(SearchStruct),
    /// Db Tasks Structure
    #[clap(subcommand)]
    Tasks(TasksStruct),
}

#[derive(Subcommand, Debug)]
pub enum TasksStruct {
    /// Manages a CSV file.
    #[clap(subcommand)]
    Csv(CsvStruct),
    /// Database related tasks
    #[clap(subcommand)]
    Database(Database),
    /// Reimports a directory based on scraper.
    #[clap(subcommand)]
    Reimport(Reimport),
}
#[derive(Debug, Parser)]
pub struct DirectoryLocation {
    pub location: String,
    pub site: String,
}

#[derive(Debug, Subcommand)]
pub enum Reimport {
    DirectoryLocation(DirectoryLocation),
}
#[derive(Debug, Subcommand)]
pub enum Database {
    #[clap(subcommand)]
    Remove(NamespaceInfo),
    #[clap(subcommand)]
    RemoveWhereNot(NamespaceInfo),
    /// Compresses the databases tag & relationships. Will add parent support soon.
    CompressDatabase,
    /// Checks the in memory DB
    CheckInMemdb,
    /// Checks the files on the filesystem.
    /// Warning is hella slow uses multithreading to make this go faster but still...
    CheckFiles,
    /// Backs up the database to a folder defined in settings.
    BackupDB,
}

/// Removes a namespace, tags & relationships from db.
#[derive(Debug, Subcommand)]
pub enum NamespaceInfo {
    /// A Namespace String to search for.
    NamespaceString(NamespaceString),
    /// A Namespace Id to search for.
    NamespaceId(NamespaceId),
}

#[derive(Debug, Parser)]
pub struct NamespaceString {
    /// Namespace String to search to remove.
    #[arg(exclusive = true, required = true)]
    pub namespace_string: String,
}

#[derive(Debug, Parser)]
pub struct NamespaceId {
    /// Namespace Id to remove.
    #[arg(exclusive = true, required = true)]
    pub namespace_id: usize,
}

#[derive(Debug, Subcommand)]
pub enum CsvStruct {
    /// Manages a CSV file.
    Csv,
}

#[derive(Subcommand, Debug)]
pub enum JobStruct {
    /// Adds a job to the system
    Add(JobAddStruct),
    /// Removes a job from the system
    Remove(JobRemovalStruct),
}

/// Holder of job adding.
#[derive(Debug, Parser)]
pub struct JobAddStruct {
    /// Webite, Setup by nickname or by url base
    #[arg(exclusive = false, required = true)]
    pub site: String,
    /// Tag query for multiple items use " " and a space to seperate tags
    #[arg(exclusive = false, required = true)]
    pub query: String,
    /// Time, special time of now for running a job now.
    #[arg(exclusive = false, required = true)]
    pub time: String,
    /// TODO need to fix this later.
    #[arg(exclusive = false, required = true)]
    pub committype: String,
}

/// Holder of job removal.
#[derive(Debug, Parser)]
pub struct JobRemovalStruct {
    /// Webite, Setup by nickname or by url base
    #[arg(exclusive = false, required = true)]
    pub site: String,
    /// Tag query for multiple items use " " and a space to seperate tags
    #[arg(exclusive = false, required = true)]
    pub query: String,
    /// Time, special time of now for running a job now.
    #[arg(exclusive = false, required = true)]
    pub time: String,
    /// TODO need to fix this later.
    #[arg(exclusive = false, required = true)]
    pub loops: String,
}

/// Search struct for parsing.
#[derive(Debug, Subcommand)]
pub enum SearchStruct {
    /// Searches By File ID.
    //#[arg(exclusive = true, required = false, long)]
    Fid(Id),
    /// Searches By Tag Id.
    //#[arg(exclusive = true, required = false, long)]
    Tid(Id),
    /// Searches By Tag name needs namespace.
    //#[arg(exclusive = true, required = false, long)]
    Tag(Tag),
    /// Searches By Hash.
    Hash(Hashy),
}
#[derive(Debug, Parser)]
pub struct Tag {
    #[arg(required = true, exclusive = true)]
    pub tag: String,
    #[arg(required = true, exclusive = true)]
    pub namespace: String,
}
#[derive(Debug, Parser)]
pub struct Id {
    #[arg(required = true, exclusive = true)]
    pub id: usize,
}
#[derive(Parser, Debug)]
pub struct Hashy {
    #[arg(required = true, exclusive = true)]
    pub hash: String,
}
