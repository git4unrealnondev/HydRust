use blurhash::encode;
use image::imageops::resize;
use image::imageops::FilterType;
use image::{self, DynamicImage};

// Default image width to scale to before encoding to blurhash
static WIDTH_IMG: u32 = 500;
static HEIGHT_IMG: u32 = 500;

// "Level of detail" of blur
static ENCODE_IMG_X: u32 = 5;
static ENCODE_IMG_Y: u32 = 5;

static PLUGIN_NAME: &str = "blurhash";
static PLUGIN_DESCRIPTION: &str = "Introduces Blurhash imaging support.";

#[path = "../../../src/scr/sharedtypes.rs"]
mod sharedtypes;

#[no_mangle]
pub fn return_info() -> sharedtypes::PluginInfo {
    let callbackvec = vec![sharedtypes::PluginCallback::OnDownload];
    sharedtypes::PluginInfo {
        name: PLUGIN_NAME.to_string(),
        description: PLUGIN_DESCRIPTION.to_string(),
        version: 1.00,
        api_version: 1.00,
        callbacks: callbackvec,
        communication: None,
    }
}
///
/// resizes img and inserts into db
///
fn downloadparse(img: DynamicImage) -> String {
    let rescale_img = resize(&img, WIDTH_IMG, HEIGHT_IMG, FilterType::Triangle);
    encode(
        ENCODE_IMG_X,
        ENCODE_IMG_Y,
        WIDTH_IMG,
        HEIGHT_IMG,
        &rescale_img.into_raw(),
    )
    .unwrap()
}

#[no_mangle]
//pub fn OnDownload(byteCursor: Cursor<Bytes>, Hash: &String, Ext: &String, datab: Arc<Mutex<database::Main>>) {
pub fn on_download(
    byte_c: &[u8],
    hash_in: &String,
    ext_in: &String,
) -> Vec<sharedtypes::DBPluginOutputEnum> {
    let mut output = Vec::new();

    let lmimg = image::load_from_memory(byte_c);
    match lmimg {
        Ok(good_lmimg) => {
            let string_blurhash = downloadparse(good_lmimg);

            let plugin_output = sharedtypes::DBPluginOutput {
                file: Some(vec![sharedtypes::PluginFileObj {
                    id: None,
                    hash: Some(hash_in.to_string()),
                    ext: Some(ext_in.to_string()),
                    location: None,
                }]),
                jobs: None,
                namespace: Some(vec![sharedtypes::DbPluginNamespace {
                    name: PLUGIN_NAME.to_string(),
                    description: Some(PLUGIN_DESCRIPTION.to_string()),
                }]),
                parents: None,
                setting: None,
                tag: Some(vec![sharedtypes::DBPluginTagOut {
                    name: string_blurhash.to_string(),
                    namespace: PLUGIN_NAME.to_string(),
                    parents: None,
                }]),
                relationship: Some(vec![sharedtypes::DbPluginRelationshipObj {
                    file_hash: hash_in.to_string(),
                    tag_name: string_blurhash,
                    tag_namespace: PLUGIN_NAME.to_string(),
                }]),
            };

            output.push(sharedtypes::DBPluginOutputEnum::Add(vec![plugin_output]));
        }
        Err(err) => {
            dbg!("Plugin: blurhash -- Failed to load: {}, {:?}", hash_in, err);
        }
    }
    output
}
