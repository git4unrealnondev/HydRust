///
/// Returns the location as a string that will store the string
///
pub fn getfinpath(location: &String, hash: &String) -> String {
    // Gets and makes folderpath.
    let final_loc = format!(
        "{}/{}{}/{}{}/{}{}",
        location,
        hash.chars().next().unwrap(),
        hash.chars().nth(1).unwrap(),
        hash.chars().nth(2).unwrap(),
        hash.chars().nth(3).unwrap(),
        hash.chars().nth(4).unwrap(),
        hash.chars().nth(5).unwrap()
    );
    std::fs::create_dir_all(&final_loc).unwrap();
    return final_loc;
}
