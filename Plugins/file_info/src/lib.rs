use std::collections::HashSet;
use std::fs::metadata;
use std::time::{Duration, UNIX_EPOCH};
use struct_iterable::Iterable;
use strum::{EnumIter, IntoEnumIterator};

#[path = "../../../src/scr/sharedtypes.rs"]
mod sharedtypes;

#[path = "../../../src/scr/intcoms/client.rs"]
mod client;

#[path = "../../../src/scr/db/helpers.rs"]
mod helpers;
static PLUGIN_NAME: &str = "File Info";
static PLUGIN_DESCRIPTION: &str = "Gets information from a file.";

#[no_mangle]
pub fn return_info() -> sharedtypes::PluginInfo {
    let callbackvec = vec![
        sharedtypes::PluginCallback::OnStart,
        sharedtypes::PluginCallback::OnDownload,
    ];
    sharedtypes::PluginInfo {
        name: PLUGIN_NAME.to_string(),
        description: PLUGIN_DESCRIPTION.to_string(),
        version: 1.00,
        api_version: 1.00,
        callbacks: callbackvec,
        communication: Some(sharedtypes::PluginSharedData {
            thread: sharedtypes::PluginThreadType::Inline,
            com_channel: Some(sharedtypes::PluginCommunicationChannel::Pipe(
                "beans".to_string(),
            )),
        }),
    }
}

#[no_mangle]
pub fn on_download(
    byte_c: &[u8],
    hash_in: &String,
    ext_in: &String,
) -> Vec<sharedtypes::DBPluginOutputEnum> {
    let mut output = Vec::new();

    let lmimg = image::load_from_memory(byte_c);
    match lmimg {
        Ok(img) => {
            let width = img.width();
            let height = img.height();
            let width_output = sharedtypes::DBPluginOutput {
                file: Some(vec![sharedtypes::PluginFileObj {
                    id: None,
                    hash: Some(hash_in.to_string()),
                    ext: Some(ext_in.to_string()),
                    location: None,
                }]),
                jobs: None,
                namespace: Some(vec![sharedtypes::DbPluginNamespace {
                    name: get_set(Supset::Width).name,
                    description: get_set(Supset::Width).description,
                }]),
                parents: None,
                setting: None,
                tag: Some(vec![sharedtypes::DBPluginTagOut {
                    name: width.to_string(),
                    namespace: get_set(Supset::Width).name,
                    parents: None,
                }]),
                relationship: Some(vec![sharedtypes::DbPluginRelationshipObj {
                    file_hash: hash_in.to_string(),
                    tag_name: width.to_string(),
                    tag_namespace: get_set(Supset::Width).name,
                }]),
            };
            let height_output = sharedtypes::DBPluginOutput {
                file: Some(vec![sharedtypes::PluginFileObj {
                    id: None,
                    hash: Some(hash_in.to_string()),
                    ext: Some(ext_in.to_string()),
                    location: None,
                }]),
                jobs: None,
                namespace: Some(vec![sharedtypes::DbPluginNamespace {
                    name: get_set(Supset::Height).name,
                    description: get_set(Supset::Height).description,
                }]),
                parents: None,
                setting: None,
                tag: Some(vec![sharedtypes::DBPluginTagOut {
                    name: height.to_string(),
                    namespace: get_set(Supset::Height).name,
                    parents: None,
                }]),
                relationship: Some(vec![sharedtypes::DbPluginRelationshipObj {
                    file_hash: hash_in.to_string(),
                    tag_name: height.to_string(),
                    tag_namespace: get_set(Supset::Height).name,
                }]),
            };

            output.push(sharedtypes::DBPluginOutputEnum::Add(vec![
                width_output,
                height_output,
            ]));
        }
        Err(_) => {
            client::log(format!("FileInfo - Couldn't parse size from: {}", hash_in));
        }
    }

    output
}

#[no_mangle]
pub fn on_start() {
    println!("From Fileinfo plugin");

    //fast_log::init(fast_log::Config::new().file("./log.txt")).unwrap();
    //log::info!("FileInfo - Commencing yak shaving{}", 0);
    println!("Fileinfo waiting");
    //check_existing_db();
    //log::info!("FileInfo - Commencing yak shaving{}", 1);
}

struct SettingInfo {
    name: String,
    description: Option<String>,
}
#[derive(EnumIter, PartialEq, Clone, Copy, Debug)]
enum Supset {
    Width,
    Height,
    ImportTime,
    Create,
    CreatorCreate,
}

fn get_set(inp: Supset) -> SettingInfo {
    match inp {
        Supset::Width => SettingInfo {
            name: "FileInfo-Parse-Width".to_string(),
            description: Some("From plugin FileInfo. The width of a file.".to_string()),
        },
        Supset::Height => SettingInfo {
            name: "FileInfo-Parse-Height".to_string(),
            description: Some("From plugin FileInfo. The height of a file.".to_string()),
        },
        Supset::ImportTime => SettingInfo {
            name: "FileInfo-Parse-ImportTime".to_string(),
            description: Some("From plugin FileInfo. The importation time of the file. Doubles as the last modified time if importing for the first time. Stored as a UNIX_EPOCH".to_string()),
        },
        Supset::Create => SettingInfo {
            name: "FileInfo-Parse-Create".to_string(),
            description: Some("From plugin FileInfo. The creation time of the file. Stored as a UNIX_EPOCH timestamp".to_string()),
        },
        Supset::CreatorCreate => SettingInfo {
            name: "FileInfo-Parse-Creator-Create".to_string(),
            description: Some("From plugin FileInfo. The creator creation time of the file. IE When the actual file was made by the creator. Sometimes this is embedded into a video file.".to_string()),
        },
    }
}

fn check_existing_db() {
    //std::thread::sleep(Duration::from_secs(1));
    //log::info!("FileInfo - ");
    //log::info!("FileInfo - Loading Required tabels");
    let table = sharedtypes::LoadDBTable::Tags;
    client::load_table(table);
    let table = sharedtypes::LoadDBTable::Relationship;
    client::load_table(table);
    let table = sharedtypes::LoadDBTable::Namespace;
    client::load_table(table);

    let table = sharedtypes::LoadDBTable::Files;
    client::load_table(table);
    let file_ids = client::file_get_list_all();
    let db_location = client::location_get();

    'mainloop: for table in Supset::iter() {
        let total = file_ids.clone();
        let mut name = client::settings_get_name(get_set(table).name);
        if let None = name {
            client::setting_add(
                get_set(table).name,
                get_set(table).description,
                None,
                Some("True".to_string()),
                true,
            );
            client::transaction_flush();
            name = client::settings_get_name(get_set(table).name);
        }
        // Continues the loop if this has already been checked.
        if let Some(nam) = &name {
            if nam.param.clone().unwrap() == "False" {
                continue 'mainloop;
            }
        }
        let ctab = TableData {
            name: get_set(table).name,
            description: get_set(table).description,
        };
        let utable = check_existing_db_table(ctab);
        let mut hutable = client::namespace_get_tagids(utable);
        let huetable = match hutable {
            None => HashSet::new(),
            Some(set) => set,
        };
        for each in huetable {
            if let Some(tags) = client::relationship_get_fileid(each) {
                println!("w {:?}", tags);
            }
            println!("w None: {}", each);
        }
        println!(
            "FileInfo - we've got {} files to parse for {}.",
            total.len(),
            get_set(table).name
        );
        client::transaction_flush();
        //let collionfreetable = total.symetric_difference(&huetable);
        //println!("FileInfo - Total files to process for this tag after dedup.");
        match table {
            Supset::Width => {
                let commit: usize = 1000;
                let mut cnt: usize = 0;
                let tabwidth = TableData {
                    name: get_set(Supset::Width).name,
                    description: get_set(Supset::Width).description,
                };
                let tabheight = TableData {
                    name: get_set(Supset::Height).name,
                    description: get_set(Supset::Height).description,
                };

                let uwidthtab = check_existing_db_table(tabwidth);
                let uheighttab = check_existing_db_table(tabheight);
                for file in file_ids.keys() {
                    let fpath = helpers::getfinpath(&db_location, &file_ids[file].hash);
                    let file_path = format!("{}/{}", fpath, &file_ids[file].hash);
                    if !std::path::Path::new(&file_path).is_file() {
                        client::log(format!("File does not exist: {}", file_path));
                        continue;
                    }
                    let img = image_dims(&file_path);
                    if let Some(pic) = img {
                        let uwidth = client::tag_add(pic.size.0.to_string(), uwidthtab, true, None);
                        let uheight =
                            client::tag_add(pic.size.1.to_string(), uheighttab, true, None);
                        client::relationship_add_db(*file, uwidth, true);
                        client::relationship_add_db(*file, uheight, true);
                        println!(
                            "FileID: {} Added Width: {} Height: {}",
                            file, pic.size.0, pic.size.1
                        );
                        cnt += 4;
                        if cnt >= commit {
                            client::transaction_flush();
                            cnt = 0;
                        }
                    }
                }
                // One last flush to ensure that everything gets written to db.
                client::transaction_flush();
            }
            Supset::Height => {}
            Supset::Create => {
                for file in file_ids.keys() {
                    let dat = get_file_data(&db_location, &file_ids[file].hash);

                    dbg!(&dat);
                    for (struct_name, stval) in dat.iter() {
                        if let Some(val) = stval.downcast_ref::<Option<usize>>() {
                            if let Some(ve) = val {
                                dbg!(&table);
                                dbg!(&struct_name, ve);
                                if struct_name == "modified" {
                                } else if struct_name == "created" {
                                }
                            }
                        }
                    }
                }
            }
            Supset::ImportTime => {}
            Supset::CreatorCreate => {}
        }
    }

    video_test();
}

///
/// Holder for data
///
struct TableData {
    name: String,
    description: Option<String>,
}

///
/// Checks and creates tables if not existing.
///
fn check_existing_db_table(table: TableData) -> usize {
    let bns = client::namespace_get(table.name.to_string());
    let uns = match bns {
        None => client::namespace_put(table.name, table.description, true),
        Some(id) => id,
    };
    client::transaction_flush();
    uns
}

///
/// This is for loading file.
///
fn get_file_data(location: &String, hash: &String) -> Meta {
    let empty_meta = Meta {
        create: None,
        modified: None,
    };

    let fpath = helpers::getfinpath(location, hash);
    let file_path = format!("{}/{}", fpath, hash);
    //println!("{}", file_path);
    let file_meta_unw = metadata(&file_path);
    match file_meta_unw {
        Ok(file_meta) => {
            if file_meta.is_file() {
                let file_created = match file_meta.created() {
                    Ok(filetime) => Some(
                        filetime
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            .try_into()
                            .unwrap(),
                    ),
                    Err(_) => None,
                };
                let file_mod = Some(
                    file_meta
                        .modified()
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .try_into()
                        .unwrap(),
                );
                return Meta {
                    create: file_created,
                    modified: file_mod,
                };
            }
        }
        Err(_) => return empty_meta,
    }
    empty_meta
}

///
///
///
fn image_dims(path: &String) -> Option<Picture> {
    use image::io::Reader;
    let reader = Reader::open(path)
        .unwrap()
        .with_guessed_format()
        .expect("Cursor io never fails")
        .into_dimensions();
    if let Ok(read_dec) = reader {
        let pic = Picture {
            size: read_dec,
            metadata: Meta {
                create: None,
                modified: None,
            },
        };
        return Some(pic);
    } else {
        dbg!("nune");
        return None;
    }
}

fn video_test() {
    use std::fs::File;
    ffmpeg::init().unwrap();

    let file = File::open("/home/linux/Downloads/BigBuckBunny.mp4").unwrap();
    match ffmpeg::format::io::input(file) {
        Ok(context) => {
            for (k, v) in context.metadata().iter() {
                println!("{}: {}", k, v);
            }
            let test = context.metadata();

            match test.get("creation_time") {
                None => {}
                Some(ayy) => {
                    let parsetime = chrono::DateTime::parse_from_rfc3339(&ayy).unwrap();
                    dbg!(&parsetime.timestamp_millis());
                }
            }
            if let Some(stream) = context.streams().best(ffmpeg::media::Type::Video) {
                println!("Best video stream index: {}", stream.index());
            }

            if let Some(stream) = context.streams().best(ffmpeg::media::Type::Audio) {
                println!("Best audio stream index: {}", stream.index());
            }

            if let Some(stream) = context.streams().best(ffmpeg::media::Type::Subtitle) {
                println!("Best subtitle stream index: {}", stream.index());
            }

            println!(
                "duration (seconds): {:?}",
                context
                    .duration()
                    .map(|d| d as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE))
            );

            for stream in context.streams() {
                println!("stream index {}:", stream.index());
                println!("\ttime_base: {:?}", stream.time_base());
                println!("\tstart_time: {:?}", stream.start_time());
                println!("\tduration (stream timebase): {:?}", stream.duration());
                println!(
                    "\tduration (seconds): {:?}",
                    stream
                        .time_base()
                        .zip(stream.duration())
                        .map(|(tb, d)| d as f64 * f64::from(tb))
                );
                println!("\tframes: {}", stream.frames());
                println!("\tdisposition: {:?}", stream.disposition());
                println!("\tdiscard: {:?}", stream.discard());
                println!("\tframe rate: {}", stream.frame_rate());

                let codec_par = stream.parameters();
                println!("\tmedium: {:?}", codec_par.medium());
                println!("\tid: {:?}", codec_par.id());

                let dec = stream.decoder().expect("Unable to open decoder");

                if codec_par.medium() == ffmpeg::media::Type::Video {
                    if let Ok(video) = dec.video() {
                        println!("\tbit_rate: {}", video.bit_rate());
                        println!("\tmax_rate: {}", video.max_bit_rate());
                        println!("\tdelay: {}", video.delay());
                        println!("\tvideo.width: {}", video.width());
                        println!("\tvideo.height: {}", video.height());
                        println!("\tvideo.format: {:?}", video.format());
                        println!("\tvideo.has_b_frames: {}", video.has_b_frames());
                        println!("\tvideo.aspect_ratio: {}", video.aspect_ratio());
                        println!("\tvideo.color_space: {:?}", video.color_space());
                        println!("\tvideo.color_range: {:?}", video.color_range());
                        println!("\tvideo.color_primaries: {:?}", video.color_primaries());
                        println!(
                            "\tvideo.color_transfer_characteristic: {:?}",
                            video.color_transfer_characteristic()
                        );
                        println!("\tvideo.chroma_location: {:?}", video.chroma_location());
                        println!("\tvideo.references: {}", video.references());
                        println!("\tvideo.intra_dc_precision: {}", video.intra_dc_precision());
                    }
                } else if codec_par.medium() == ffmpeg::media::Type::Audio {
                    if let Ok(audio) = dec.audio() {
                        println!("\tbit_rate: {}", audio.bit_rate());
                        println!("\tmax_rate: {}", audio.max_bit_rate());
                        println!("\tdelay: {}", audio.delay());
                        println!("\taudio.sample_rate: {}", audio.sample_rate());
                        println!("\taudio.channels: {}", audio.channels());
                        println!("\taudio.format: {:?}", audio.format());
                        println!("\taudio.frames: {}", audio.frames());
                        println!("\taudio.align: {}", audio.align());
                        println!("\taudio.channel_layout: {:?}", audio.channel_layout());
                    }
                }
            }
        }

        Err(error) => println!("error: {}", error),
    }
}

#[derive(Debug, Iterable)]
struct Picture {
    size: (u32, u32),
    metadata: Meta,
}

#[derive(Debug, Iterable)]
struct Video {
    size: (u32, u32),
    duration: f64,
    metadata: Meta,
}

enum Parseables {
    Picture(Picture),
    Video(Video),
}
#[derive(Debug, Iterable)]
struct Meta {
    create: Option<usize>,
    modified: Option<usize>,
}
