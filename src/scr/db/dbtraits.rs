use crate::sharedtypes;

use std::collections::{HashMap, HashSet};

pub trait DBTraits {
    ///
    /// Puts a parent into db
    ///
    fn parents_put(
        &mut self,
        tag_namespace_id: usize,
        tag_id: usize,
        relate_tag_id: usize,
        relate_namespace_id: usize,
    ) -> usize;
    ///
    /// Checks if relationship exists
    ///
    fn relationship_get(&self, file: &usize, tag: &usize) -> bool;
    ///
    /// Adds relationship between db
    ///
    #[inline(always)]
    fn relationship_add(&mut self, file: usize, tag: usize);

    ///
    /// Adds a job to the in db
    ///
    fn jobs_add(&mut self, job: sharedtypes::DbJobsObj);
    ///
    /// Get max of job
    ///
    fn jobs_get_max(&self) -> &usize;
    ///
    /// Get all jobs
    ///
    fn jobs_get_all(&self) -> &HashMap<usize, sharedtypes::DbJobsObj>;
    ///
    /// Removes parent's from internal db based on tag id
    ///

    fn parents_remove(&mut self, tag_id: &usize) -> HashSet<(usize, usize)>;
    ///
    /// Returns the list of tags assicated with relationship
    ///
    fn parents_tag_get(&self, tag_id: &usize) -> Option<&HashSet<usize>>;
    ///
    /// Returns the list of relationships assicated with tag
    ///
    fn parents_rel_get(&self, relate_tag_id: &usize) -> Option<&HashSet<usize>>;
    ///
    /// checks if parents exist
    ///
    fn parents_get(&self, parent: &sharedtypes::DbParentsObj) -> Option<&usize>;
    ///
    /// Adds a tag into db
    ///
    fn tags_put(&mut self, tag_info: sharedtypes::DbTagNNS, id: Option<usize>) -> usize;
    ///
    /// Returns the max id of namespaces
    ///
    fn namespace_get_max(&self) -> usize;
    ///
    /// Deletes a namespace from db
    ///
    fn namespace_delete(&mut self, namepsace_id: &usize);
    ///
    /// Inserts namespace into db
    ///
    fn namespace_put(&mut self, namespace_obj: sharedtypes::DbNamespaceObj) -> usize;
    ///
    /// Returns the file id's in db
    ///
    fn file_get_list_id(&self) -> HashSet<usize>;

    ///
    /// get's file if from db hash
    ///
    fn file_get_hash(&self, hash: &String) -> Option<&usize>;
    ///
    /// Returns a file based on ID
    ///
    fn file_get_id(&self, id: &usize) -> Option<&sharedtypes::DbFileObj>;
    ///
    /// inserts file into db returns file id
    ///
    fn file_put(&mut self, file: sharedtypes::DbFileObj) -> usize;
    ///
    /// Resets the tag counter to 0.
    ///
    fn tags_max_reset(&mut self);
    ///
    /// Returns the max id in db
    ///
    fn tags_max_return(&self) -> &usize;
    ///
    /// Removes relationship from db
    ///
    fn relationship_remove(&mut self, file_id: &usize, tag_id: &usize);
    ///
    /// Removes tag from db.
    ///
    fn tag_remove(&mut self, id: &usize) -> Option<()>;
    ///
    ///
    ///
    fn tags_get_list_id(&self) -> HashSet<usize>;
    ///
    /// Clears inmemdb relationships structures
    ///
    fn relationships_clear(&mut self);
    ///
    /// Clears inmemdb parents structures
    ///
    fn parents_clear(&mut self);
    ///
    /// Clears inmemdb tags structures
    ///
    fn tags_clear(&mut self);
    ///
    /// Returns the tag from id
    ///
    fn tags_get_data(&self, id: &usize) -> Option<&sharedtypes::DbTagNNS>;
    ///
    /// Returns the tag id of the nns
    ///
    fn tags_get_id(&self, tagobj: &sharedtypes::DbTagNNS) -> Option<&usize>;
    ///
    /// relationship gets only one fileid
    ///
    fn relationship_get_one_fileid(&self, tag: &usize) -> Option<&usize>;
    ///
    /// Returns a list of tag id's based on a file id
    ///
    fn relationship_get_tagid(&self, file: &usize) -> Option<&HashSet<usize>>;
    ///
    /// Returns a list of file id's based on a tag id.
    ///
    fn relationship_get_fileid(&self, tag: &usize) -> Option<&HashSet<usize>>;
    ///
    /// Returns tag id's based on namespace id.
    ///
    fn namespace_get_tagids(&self, id: &usize) -> Option<&HashSet<usize>>;
    ///
    /// Retuns raw namespace id's
    ///
    fn namespace_keys(&self) -> Vec<usize>;
    ///
    /// Returns namespace obj by id
    ///
    fn namespace_id_get(&self, id: &usize) -> Option<&sharedtypes::DbNamespaceObj>;
    ///
    /// Returns namespace id by string
    ///
    fn namespace_get(&self, inp: &String) -> Option<&usize>;
    ///
    /// Returns tag by ID.
    ///
    fn settings_get_name(&self, name: &String) -> Option<&sharedtypes::DbSettingObj>;
    ///
    /// Adds jobs into internal db.
    ///
    fn jobref_new(&mut self, job: sharedtypes::DbJobsObj);
    ///
    /// Adds setting into internal DB.
    ///
    fn settings_add(
        &mut self,
        name: String,
        pretty: Option<String>,
        num: Option<usize>,
        param: Option<String>,
    );
}
