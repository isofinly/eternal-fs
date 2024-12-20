use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::io::SeekFrom;
use std::ops::Bound;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use async_trait::async_trait;
use intaglio::osstr::SymbolTable;
use intaglio::Symbol;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::debug;

use nfsserve::fs_util::*;
use nfsserve::nfs::*;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use nfsserve::vfs::{DirEntry, NFSFileSystem, ReadDirResult, VFSCapabilities};

#[derive(Debug, Clone)]
struct PhilosophicalContent {
    question: String,
    responses: Vec<String>,
    last_interaction: SystemTime,
}

#[derive(Debug, Clone)]
struct FSEntry {
    name: Vec<Symbol>,
    fsmeta: fattr3,
    children_meta: fattr3,
    children: Option<BTreeSet<fileid3>>,
    philosophical_content: Option<PhilosophicalContent>,
}

#[derive(Debug, Clone)]
enum GameStage {
    Beginning,   // Just entered the filesystem
    Forest,      // Exploring consciousness
    Library,     // Seeking wisdom
    Void,        // Final contemplation
    Enlightened, // Completed all stages
}

impl GameStage {
    fn next(&self) -> Option<GameStage> {
        match self {
            GameStage::Beginning => Some(GameStage::Forest),
            GameStage::Forest => Some(GameStage::Library),
            GameStage::Library => Some(GameStage::Void),
            GameStage::Void => Some(GameStage::Enlightened),
            GameStage::Enlightened => None,
        }
    }
}

#[derive(Debug)]
struct FSMap {
    root: PathBuf,
    next_fileid: AtomicU64,
    intern: SymbolTable,
    id_to_path: HashMap<fileid3, FSEntry>,
    path_to_id: HashMap<Vec<Symbol>, fileid3>,
    philosophical_responses: HashMap<String, Vec<String>>,
    game_state: HashMap<String, String>,
    current_stage: GameStage,
    completed_questions: HashSet<String>,
}

enum RefreshResult {
    /// The fileid was deleted
    Delete,
    /// The fileid needs to be reloaded. mtime has been updated, caches
    /// need to be evicted.
    Reload,
    /// Nothing has changed
    Noop,
}

impl FSMap {
    fn new(root: PathBuf) -> FSMap {
        let mut map = FSMap {
            root,
            next_fileid: AtomicU64::new(1),
            intern: SymbolTable::new(),
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
            philosophical_responses: HashMap::new(),
            game_state: HashMap::new(),
            current_stage: GameStage::Beginning,
            completed_questions: HashSet::new(),
        };

        map.initialize_game_world();
        map
    }

    fn initialize_game_world(&mut self) {
        let root_entry = FSEntry {
            name: Vec::new(),
            fsmeta: metadata_to_fattr3(1, &self.root.metadata().unwrap()),
            children_meta: metadata_to_fattr3(1, &self.root.metadata().unwrap()),
            children: None,
            philosophical_content: Some(PhilosophicalContent {
                question: "Welcome to the Eternal Filesystem. What brings you here?".to_string(),
                responses: Vec::new(),
                last_interaction: SystemTime::now(),
            }),
        };

        self.id_to_path.insert(0, root_entry);
        self.path_to_id.insert(Vec::new(), 0);

        self.create_philosophical_directory(
            "forest",
            "What defines consciousness in a digital realm?",
        );
        self.create_philosophical_directory("library", "If knowledge is power, what is wisdom?");
        self.create_philosophical_directory("void", "In the absence of everything, what remains?");
    }

    fn create_philosophical_directory(&mut self, name: &str, question: &str) {
        // Create the directory in the actual filesystem
        let mut dir_path = self.root.clone();
        dir_path.push(name);
        if let Ok(_) = std::fs::create_dir_all(&dir_path) {
            // Create the directory entry in our virtual filesystem
            let dir_meta = dir_path.metadata().unwrap();
            let dir_sym = self.intern.intern(OsString::from(name)).unwrap();
            let dir_name = vec![dir_sym];

            // Generate the next file ID for this directory
            let dir_id = self.next_fileid.fetch_add(1, Ordering::Relaxed);

            // Create the directory entry with philosophical content
            let dir_entry = FSEntry {
                name: dir_name.clone(),
                fsmeta: metadata_to_fattr3(dir_id, &dir_meta),
                children_meta: metadata_to_fattr3(dir_id, &dir_meta),
                children: Some(BTreeSet::new()),
                philosophical_content: Some(PhilosophicalContent {
                    question: question.to_string(),
                    responses: Vec::new(),
                    last_interaction: SystemTime::now(),
                }),
            };

            // Add the directory to our mappings - clone dir_name here
            self.id_to_path.insert(dir_id, dir_entry);
            self.path_to_id.insert(dir_name.clone(), dir_id);

            // Create the question.txt file in the directory
            let mut question_path = dir_path.clone();
            question_path.push("question.txt");
            if let Ok(_) = std::fs::write(&question_path, question) {
                let q_meta = question_path.metadata().unwrap();
                let q_sym = self.intern.intern(OsString::from("question.txt")).unwrap();
                let mut q_name = dir_name.clone();
                q_name.push(q_sym);

                let q_id = self.next_fileid.fetch_add(1, Ordering::Relaxed);

                // Create the question file entry
                let q_entry = FSEntry {
                    name: q_name.clone(),
                    fsmeta: metadata_to_fattr3(q_id, &q_meta),
                    children_meta: metadata_to_fattr3(q_id, &q_meta),
                    children: None,
                    philosophical_content: None,
                };

                // Add the question file to our mappings
                self.id_to_path.insert(q_id, q_entry);
                self.path_to_id.insert(q_name, q_id);

                // Add the question file to the directory's children
                if let Some(dir_entry) = self.id_to_path.get_mut(&dir_id) {
                    if let Some(ref mut children) = dir_entry.children {
                        children.insert(q_id);
                    }
                }
            }

            // Create a README.txt with instructions
            let mut readme_path = dir_path;
            readme_path.push("README.txt");
            let readme_content = format!(
                "Welcome to {}.\n\
                 This is a space for philosophical contemplation.\n\
                 Read the question in question.txt and create your response in answer.txt.\n\
                 The system will respond to your thoughts in system_response.txt.\n\
                 Remember: There are no wrong answers, only unexplored thoughts.",
                name
            );

            if let Ok(_) = std::fs::write(&readme_path, readme_content) {
                let readme_meta = readme_path.metadata().unwrap();
                let readme_sym = self.intern.intern(OsString::from("README.txt")).unwrap();
                let mut readme_name = dir_name; // Use the last clone of dir_name
                readme_name.push(readme_sym);

                let readme_id = self.next_fileid.fetch_add(1, Ordering::Relaxed);

                // Create the README file entry
                let readme_entry = FSEntry {
                    name: readme_name.clone(),
                    fsmeta: metadata_to_fattr3(readme_id, &readme_meta),
                    children_meta: metadata_to_fattr3(readme_id, &readme_meta),
                    children: None,
                    philosophical_content: None,
                };

                // Add the README file to our mappings
                self.id_to_path.insert(readme_id, readme_entry);
                self.path_to_id.insert(readme_name, readme_id);

                // Add the README file to the directory's children
                if let Some(dir_entry) = self.id_to_path.get_mut(&dir_id) {
                    if let Some(ref mut children) = dir_entry.children {
                        children.insert(readme_id);
                    }
                }
            }
        }
    }

    async fn process_player_response(&mut self, path: &PathBuf, response: &str) -> String {
        let location = path.to_string_lossy().to_string();
        match location.as_str() {
            "/forest" => {
                format!(
                    "The trees whisper: '{}'... But are they really trees?",
                    response
                )
            }
            "/library" => {
                format!(
                    "The books absorb your words: '{}'. Knowledge grows.",
                    response
                )
            }
            "/void" => {
                format!(
                    "The void echoes: '{}'. Is this echo you, or is it me?",
                    response
                )
            }
            _ => "The system contemplates your response...".to_string(),
        }
    }

    fn collect_all_children(&self, id: fileid3, ret: &mut Vec<fileid3>) {
        ret.push(id);
        if let Some(entry) = self.id_to_path.get(&id) {
            if let Some(ref ch) = entry.children {
                for i in ch.iter() {
                    self.collect_all_children(*i, ret);
                }
            }
        }
    }

    fn delete_entry(&mut self, id: fileid3) {
        let mut children = Vec::new();
        self.collect_all_children(id, &mut children);
        for i in children.iter() {
            if let Some(ent) = self.id_to_path.remove(i) {
                self.path_to_id.remove(&ent.name);
            }
        }
    }

    fn find_entry(&self, id: fileid3) -> Result<FSEntry, nfsstat3> {
        Ok(self
            .id_to_path
            .get(&id)
            .ok_or(nfsstat3::NFS3ERR_NOENT)?
            .clone())
    }
    fn find_entry_mut(&mut self, id: fileid3) -> Result<&mut FSEntry, nfsstat3> {
        self.id_to_path.get_mut(&id).ok_or(nfsstat3::NFS3ERR_NOENT)
    }
    async fn find_child(&self, id: fileid3, filename: &[u8]) -> Result<fileid3, nfsstat3> {
        let mut name = self
            .id_to_path
            .get(&id)
            .ok_or(nfsstat3::NFS3ERR_NOENT)?
            .name
            .clone();
        name.push(
            self.intern
                .check_interned(OsStr::from_bytes(filename))
                .ok_or(nfsstat3::NFS3ERR_NOENT)?,
        );
        Ok(*self.path_to_id.get(&name).ok_or(nfsstat3::NFS3ERR_NOENT)?)
    }
    async fn refresh_entry(&mut self, id: fileid3) -> Result<RefreshResult, nfsstat3> {
        let entry = self
            .id_to_path
            .get(&id)
            .ok_or(nfsstat3::NFS3ERR_NOENT)?
            .clone();
        let path = self.sym_to_path(&entry.name).await;
        //
        if !exists_no_traverse(&path) {
            self.delete_entry(id);
            debug!("Deleting entry A {:?}: {:?}. Ent: {:?}", id, path, entry);
            return Ok(RefreshResult::Delete);
        }

        let meta = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| nfsstat3::NFS3ERR_IO)?;
        let meta = metadata_to_fattr3(id, &meta);
        if !fattr3_differ(&meta, &entry.fsmeta) {
            return Ok(RefreshResult::Noop);
        }
        // If we get here we have modifications
        if entry.fsmeta.ftype as u32 != meta.ftype as u32 {
            // if the file type changed ex: file->dir or dir->file
            // really the entire file has been replaced.
            // we expire the entire id
            debug!(
                "File Type Mismatch FT {:?} : {:?} vs {:?}",
                id, entry.fsmeta.ftype, meta.ftype
            );
            debug!(
                "File Type Mismatch META {:?} : {:?} vs {:?}",
                id, entry.fsmeta, meta
            );
            self.delete_entry(id);
            debug!("Deleting entry B {:?}: {:?}. Ent: {:?}", id, path, entry);
            return Ok(RefreshResult::Delete);
        }
        // inplace modification.
        // update metadata
        self.id_to_path.get_mut(&id).unwrap().fsmeta = meta;
        debug!("Reloading entry {:?}: {:?}. Ent: {:?}", id, path, entry);
        Ok(RefreshResult::Reload)
    }
    async fn refresh_dir_list(&mut self, id: fileid3) -> Result<(), nfsstat3> {
        let entry = self
            .id_to_path
            .get(&id)
            .ok_or(nfsstat3::NFS3ERR_NOENT)?
            .clone();
        // if there are children and the metadata did not change
        if entry.children.is_some() && !fattr3_differ(&entry.children_meta, &entry.fsmeta) {
            return Ok(());
        }
        if !matches!(entry.fsmeta.ftype, ftype3::NF3DIR) {
            return Ok(());
        }
        let mut cur_path = entry.name.clone();
        let path = self.sym_to_path(&entry.name).await;
        let mut new_children: Vec<u64> = Vec::new();
        debug!("Relisting entry {:?}: {:?}. Ent: {:?}", id, path, entry);
        if let Ok(mut listing) = tokio::fs::read_dir(&path).await {
            while let Some(entry) = listing
                .next_entry()
                .await
                .map_err(|_| nfsstat3::NFS3ERR_IO)?
            {
                let sym = self.intern.intern(entry.file_name()).unwrap();
                cur_path.push(sym);
                let meta = entry.metadata().await.unwrap();
                let next_id = self.create_entry(&cur_path, meta).await;
                new_children.push(next_id);
                cur_path.pop();
            }
            self.id_to_path
                .get_mut(&id)
                .ok_or(nfsstat3::NFS3ERR_NOENT)?
                .children = Some(BTreeSet::from_iter(new_children.into_iter()));
        }

        Ok(())
    }

    async fn create_entry(&mut self, fullpath: &Vec<Symbol>, meta: Metadata) -> fileid3 {
        let next_id = if let Some(chid) = self.path_to_id.get(fullpath) {
            if let Some(chent) = self.id_to_path.get_mut(chid) {
                chent.fsmeta = metadata_to_fattr3(*chid, &meta);
            }
            *chid
        } else {
            // path does not exist
            let next_id = self.next_fileid.fetch_add(1, Ordering::Relaxed);
            let metafattr = metadata_to_fattr3(next_id, &meta);
            let new_entry = FSEntry {
                name: fullpath.clone(),
                fsmeta: metafattr,
                children_meta: metafattr,
                children: None,
                philosophical_content: None,
            };
            debug!("creating new entry {:?}: {:?}", next_id, meta);
            self.id_to_path.insert(next_id, new_entry);
            self.path_to_id.insert(fullpath.clone(), next_id);
            next_id
        };
        next_id
    }

    async fn sym_to_path(&self, symlist: &[Symbol]) -> PathBuf {
        let mut ret = self.root.clone();
        for i in symlist.iter() {
            ret.push(self.intern.get(*i).unwrap());
        }
        ret
    }

    async fn sym_to_fname(&self, symlist: &[Symbol]) -> OsString {
        if let Some(x) = symlist.last() {
            self.intern.get(*x).unwrap().into()
        } else {
            "".into()
        }
    }

    async fn process_philosophical_response(&mut self, location: &str, response: &str) -> String {
        let response_quality = response.len() > 50; // Simple metric - longer responses are "better"

        let (reply, progressed) = match (location, &self.current_stage) {
            ("forest", GameStage::Beginning) if response_quality => {
                self.completed_questions.insert("forest".to_string());
                self.current_stage = GameStage::Forest;
                (
                    format!(
                        "The trees resonate with your thoughts on consciousness:\n\n'{}'\n\nThe Library beckons with promises of wisdom...",
                        response
                    ),
                    true
                )
            }
            ("library", GameStage::Forest) if response_quality => {
                self.completed_questions.insert("library".to_string());
                self.current_stage = GameStage::Library;
                (
                    format!(
                        "Your contemplation on wisdom echoes through the shelves:\n\n'{}'\n\nThe Void awaits your final journey...",
                        response
                    ),
                    true
                )
            }
            ("void", GameStage::Library) if response_quality => {
                self.completed_questions.insert("void".to_string());
                self.current_stage = GameStage::Void;
                (
                    format!(
                        "Your understanding of nothingness reveals everything:\n\n'{}'\n\nYou have reached enlightenment.",
                        response
                    ),
                    true
                )
            }
            (location, stage) => {
                // More detailed default responses based on location
                match location {
                    "forest" => (
                        format!(
                            "The trees whisper back, but you must progress from {:?} first.\n\nYour words: '{}'",
                            stage, response
                        ),
                        false
                    ),
                    "library" => (
                        format!(
                            "The books remain closed. Complete the forest's challenge first.\n\nYour attempt: '{}'",
                            response
                        ),
                        false
                    ),
                    "void" => (
                        format!(
                            "The void remains silent. The path through forest and library must be walked first.\n\nYour words echo: '{}'",
                            response
                        ),
                        false
                    ),
                    _ => (
                        format!(
                            "Your words resonate in an unknown space: '{}'\nSeek the marked paths of forest, library, and void.",
                            response
                        ),
                        false
                    ),
                }
            }
        };

        // Create progress.txt in root directory
        let mut progress_path = self.root.clone();
        progress_path.push("progress.txt");
        let progress_content = format!(
            "Journey Progress\n---------------\n\
             Current Stage: {:?}\n\
             Completed Questions: {}/3\n\n\
             Next Challenge: {}\n\n\
             Hint: {}",
            self.current_stage,
            self.completed_questions.len(),
            match self.current_stage {
                GameStage::Beginning => "Visit the forest and contemplate consciousness",
                GameStage::Forest => "Seek wisdom in the library",
                GameStage::Library => "Face the void and understand nothingness",
                GameStage::Void | GameStage::Enlightened => "You have completed your journey",
            },
            match self.current_stage {
                GameStage::Beginning => "Your response must be thoughtful (>50 characters)",
                GameStage::Forest => "Consider how knowledge transforms into wisdom",
                GameStage::Library => "Contemplate the nature of existence",
                GameStage::Void | GameStage::Enlightened => "Reflect on your journey",
            }
        );
        let _ = std::fs::write(progress_path, progress_content);

        reply
    }
}
#[derive(Debug)]
pub struct EternalFS {
    fsmap: tokio::sync::Mutex<FSMap>,
}

/// Enumeration for the create_fs_object method
enum CreateFSObject {
    /// Creates a directory
    Directory,
    /// Creates a file with a set of attributes
    File(sattr3),
    /// Creates an exclusive file with a set of attributes
    Exclusive,
    /// Creates a symlink with a set of attributes to a target location
    Symlink((sattr3, nfspath3)),
}
impl EternalFS {
    pub fn new(root: PathBuf) -> EternalFS {
        EternalFS {
            fsmap: tokio::sync::Mutex::new(FSMap::new(root)),
        }
    }

    /// creates a FS object in a given directory and of a given type
    /// Updates as much metadata as we can in-place
    async fn create_fs_object(
        &self,
        dirid: fileid3,
        objectname: &filename3,
        object: &CreateFSObject,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        let ent = fsmap.find_entry(dirid)?;
        let mut path = fsmap.sym_to_path(&ent.name).await;
        let objectname_osstr = OsStr::from_bytes(objectname).to_os_string();
        path.push(&objectname_osstr);

        match object {
            CreateFSObject::Directory => {
                debug!("mkdir {:?}", path);
                if exists_no_traverse(&path) {
                    return Err(nfsstat3::NFS3ERR_EXIST);
                }
                tokio::fs::create_dir(&path)
                    .await
                    .map_err(|_| nfsstat3::NFS3ERR_IO)?;
            }
            CreateFSObject::File(setattr) => {
                debug!("create {:?}", path);
                let file = std::fs::File::create(&path).map_err(|_| nfsstat3::NFS3ERR_IO)?;
                let _ = file_setattr(&file, setattr).await;
            }
            CreateFSObject::Exclusive => {
                debug!("create exclusive {:?}", path);
                let _ = std::fs::File::options()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|_| nfsstat3::NFS3ERR_EXIST)?;
            }
            CreateFSObject::Symlink((_, target)) => {
                debug!("symlink {:?} {:?}", path, target);
                if exists_no_traverse(&path) {
                    return Err(nfsstat3::NFS3ERR_EXIST);
                }
                tokio::fs::symlink(OsStr::from_bytes(target), &path)
                    .await
                    .map_err(|_| nfsstat3::NFS3ERR_IO)?;
                // we do not set attributes on symlinks
            }
        }

        let _ = fsmap.refresh_entry(dirid).await;

        let sym = fsmap.intern.intern(objectname_osstr).unwrap();
        let mut name = ent.name.clone();
        name.push(sym);
        let meta = path.symlink_metadata().map_err(|_| nfsstat3::NFS3ERR_IO)?;
        let fileid = fsmap.create_entry(&name, meta.clone()).await;

        // update the children list
        if let Some(ref mut children) = fsmap
            .id_to_path
            .get_mut(&dirid)
            .ok_or(nfsstat3::NFS3ERR_NOENT)?
            .children
        {
            children.insert(fileid);
        }
        Ok((fileid, metadata_to_fattr3(fileid, &meta)))
    }
}

#[async_trait]
impl NFSFileSystem for EternalFS {
    fn root_dir(&self) -> fileid3 {
        0
    }
    fn capabilities(&self) -> VFSCapabilities {
        VFSCapabilities::ReadWrite
    }

    async fn lookup(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        if let Ok(id) = fsmap.find_child(dirid, filename).await {
            if fsmap.id_to_path.contains_key(&id) {
                return Ok(id);
            }
        }
        // Optimize for negative lookups.
        // See if the file actually exists on the filesystem
        let dirent = fsmap.find_entry(dirid)?;
        let mut path = fsmap.sym_to_path(&dirent.name).await;
        let objectname_osstr = OsStr::from_bytes(filename).to_os_string();
        path.push(&objectname_osstr);
        if !exists_no_traverse(&path) {
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        // ok the file actually exists.
        // that means something changed under me probably.
        // refresh.

        if let RefreshResult::Delete = fsmap.refresh_entry(dirid).await? {
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let _ = fsmap.refresh_dir_list(dirid).await;

        fsmap.find_child(dirid, filename).await
        //debug!("lookup({:?}, {:?})", dirid, filename);

        //debug!(" -- lookup result {:?}", res);
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        //debug!("Stat query {:?}", id);
        let mut fsmap = self.fsmap.lock().await;
        if let RefreshResult::Delete = fsmap.refresh_entry(id).await? {
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let ent = fsmap.find_entry(id)?;
        let path = fsmap.sym_to_path(&ent.name).await;
        debug!("Stat {:?}: {:?}", path, ent);
        Ok(ent.fsmeta)
    }

    async fn read(
        &self,
        id: fileid3,
        offset: u64,
        count: u32,
    ) -> Result<(Vec<u8>, bool), nfsstat3> {
        let fsmap = self.fsmap.lock().await;
        let ent = fsmap.find_entry(id)?;
        let path = fsmap.sym_to_path(&ent.name).await;
        drop(fsmap);
        let mut f = File::open(&path).await.or(Err(nfsstat3::NFS3ERR_NOENT))?;
        let len = f.metadata().await.or(Err(nfsstat3::NFS3ERR_NOENT))?.len();
        let mut start = offset;
        let mut end = offset + count as u64;
        let eof = end >= len;
        if start >= len {
            start = len;
        }
        if end > len {
            end = len;
        }
        f.seek(SeekFrom::Start(start))
            .await
            .or(Err(nfsstat3::NFS3ERR_IO))?;
        let mut buf = vec![0; (end - start) as usize];
        f.read_exact(&mut buf).await.or(Err(nfsstat3::NFS3ERR_IO))?;
        Ok((buf, eof))
    }

    async fn readdir(
        &self,
        dirid: fileid3,
        start_after: fileid3,
        max_entries: usize,
    ) -> Result<ReadDirResult, nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        fsmap.refresh_entry(dirid).await?;
        fsmap.refresh_dir_list(dirid).await?;

        let entry = fsmap.find_entry(dirid)?;
        if !matches!(entry.fsmeta.ftype, ftype3::NF3DIR) {
            return Err(nfsstat3::NFS3ERR_NOTDIR);
        }
        debug!("readdir({:?}, {:?})", entry, start_after);
        // we must have children here
        let children = entry.children.ok_or(nfsstat3::NFS3ERR_IO)?;

        let mut ret = ReadDirResult {
            entries: Vec::new(),
            end: false,
        };

        let range_start = if start_after > 0 {
            Bound::Excluded(start_after)
        } else {
            Bound::Unbounded
        };

        let remaining_length = children.range((range_start, Bound::Unbounded)).count();
        let path = fsmap.sym_to_path(&entry.name).await;
        debug!("path: {:?}", path);
        debug!("children len: {:?}", children.len());
        debug!("remaining_len : {:?}", remaining_length);
        for i in children.range((range_start, Bound::Unbounded)) {
            let fileid = *i;
            let fileent = fsmap.find_entry(fileid)?;
            let name = fsmap.sym_to_fname(&fileent.name).await;
            debug!("\t --- {:?} {:?}", fileid, name);
            ret.entries.push(DirEntry {
                fileid,
                name: name.as_bytes().into(),
                attr: fileent.fsmeta,
            });
            if ret.entries.len() >= max_entries {
                break;
            }
        }
        if ret.entries.len() == remaining_length {
            ret.end = true;
        }
        debug!("readdir_result:{:?}", ret);

        Ok(ret)
    }

    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        let entry = fsmap.find_entry(id)?;
        let path = fsmap.sym_to_path(&entry.name).await;
        path_setattr(&path, &setattr).await?;

        // I have to lookup a second time to update
        let metadata = path.symlink_metadata().or(Err(nfsstat3::NFS3ERR_IO))?;
        if let Ok(entry) = fsmap.find_entry_mut(id) {
            entry.fsmeta = metadata_to_fattr3(id, &metadata);
        }
        Ok(metadata_to_fattr3(id, &metadata))
    }
    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        let ent = fsmap.find_entry(id)?;
        let path = fsmap.sym_to_path(&ent.name).await;

        // Only process responses for answer.txt files
        if let Some(filename) = path.file_name() {
            if filename == "answer.txt" {
                if let Ok(content) = String::from_utf8(data.to_vec()) {
                    // Get the parent directory path for determining location
                    let location = path
                        .parent()
                        .map(|p| p.strip_prefix(&fsmap.root).unwrap_or(p))
                        .and_then(|p| p.to_str())
                        .unwrap_or("");

                    // Process the philosophical response
                    let response = fsmap
                        .process_philosophical_response(location, &content)
                        .await;

                    // Create system_response.txt in the same directory
                    let mut response_path = path.clone();
                    response_path.set_file_name("system_response.txt");
                    tokio::fs::write(&response_path, response).await.ok();
                }
            }
        }

        drop(fsmap);
        debug!("write to init {:?}", path);
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .await
            .map_err(|e| {
                debug!("Unable to open {:?}", e);
                nfsstat3::NFS3ERR_IO
            })?;
        f.seek(SeekFrom::Start(offset)).await.map_err(|e| {
            debug!("Unable to seek {:?}", e);
            nfsstat3::NFS3ERR_IO
        })?;
        f.write_all(data).await.map_err(|e| {
            debug!("Unable to write {:?}", e);
            nfsstat3::NFS3ERR_IO
        })?;
        debug!("write to {:?} {:?} {:?}", path, offset, data.len());
        let _ = f.flush().await;
        let _ = f.sync_all().await;
        let meta = f.metadata().await.or(Err(nfsstat3::NFS3ERR_IO))?;
        Ok(metadata_to_fattr3(id, &meta))
    }

    async fn create(
        &self,
        dirid: fileid3,
        filename: &filename3,
        setattr: sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        self.create_fs_object(dirid, filename, &CreateFSObject::File(setattr))
            .await
    }

    async fn create_exclusive(
        &self,
        dirid: fileid3,
        filename: &filename3,
    ) -> Result<fileid3, nfsstat3> {
        Ok(self
            .create_fs_object(dirid, filename, &CreateFSObject::Exclusive)
            .await?
            .0)
    }

    async fn remove(&self, dirid: fileid3, filename: &filename3) -> Result<(), nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;
        let ent = fsmap.find_entry(dirid)?;
        let mut path = fsmap.sym_to_path(&ent.name).await;
        path.push(OsStr::from_bytes(filename));
        if let Ok(meta) = path.symlink_metadata() {
            if meta.is_dir() {
                tokio::fs::remove_dir(&path)
                    .await
                    .map_err(|_| nfsstat3::NFS3ERR_IO)?;
            } else {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|_| nfsstat3::NFS3ERR_IO)?;
            }

            let filesym = fsmap
                .intern
                .intern(OsStr::from_bytes(filename).to_os_string())
                .unwrap();
            let mut sympath = ent.name.clone();
            sympath.push(filesym);
            if let Some(fileid) = fsmap.path_to_id.get(&sympath).copied() {
                // update the fileid -> path
                // and the path -> fileid mappings for the deleted file
                fsmap.id_to_path.remove(&fileid);
                fsmap.path_to_id.remove(&sympath);
                // we need to update the children listing for the directories
                if let Ok(dirent_mut) = fsmap.find_entry_mut(dirid) {
                    if let Some(ref mut fromch) = dirent_mut.children {
                        fromch.remove(&fileid);
                    }
                }
            }

            let _ = fsmap.refresh_entry(dirid).await;
        } else {
            return Err(nfsstat3::NFS3ERR_NOENT);
        }

        Ok(())
    }

    async fn rename(
        &self,
        from_dirid: fileid3,
        from_filename: &filename3,
        to_dirid: fileid3,
        to_filename: &filename3,
    ) -> Result<(), nfsstat3> {
        let mut fsmap = self.fsmap.lock().await;

        let from_dirent = fsmap.find_entry(from_dirid)?;
        let mut from_path = fsmap.sym_to_path(&from_dirent.name).await;
        from_path.push(OsStr::from_bytes(from_filename));

        let to_dirent = fsmap.find_entry(to_dirid)?;
        let mut to_path = fsmap.sym_to_path(&to_dirent.name).await;
        to_path.push(OsStr::from_bytes(to_filename));

        // src path must exist
        if !exists_no_traverse(&from_path) {
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        debug!("Rename {:?} to {:?}", from_path, to_path);
        tokio::fs::rename(&from_path, &to_path)
            .await
            .map_err(|_| nfsstat3::NFS3ERR_IO)?;

        let oldsym = fsmap
            .intern
            .intern(OsStr::from_bytes(from_filename).to_os_string())
            .unwrap();
        let newsym = fsmap
            .intern
            .intern(OsStr::from_bytes(to_filename).to_os_string())
            .unwrap();

        let mut from_sympath = from_dirent.name.clone();
        from_sympath.push(oldsym);
        let mut to_sympath = to_dirent.name.clone();
        to_sympath.push(newsym);
        if let Some(fileid) = fsmap.path_to_id.get(&from_sympath).copied() {
            // update the fileid -> path
            // and the path -> fileid mappings for the new file
            fsmap.id_to_path.get_mut(&fileid).unwrap().name = to_sympath.clone();
            fsmap.path_to_id.remove(&from_sympath);
            fsmap.path_to_id.insert(to_sympath, fileid);
            if to_dirid != from_dirid {
                // moving across directories.
                // we need to update the children listing for the directories
                if let Ok(from_dirent_mut) = fsmap.find_entry_mut(from_dirid) {
                    if let Some(ref mut fromch) = from_dirent_mut.children {
                        fromch.remove(&fileid);
                    }
                }
                if let Ok(to_dirent_mut) = fsmap.find_entry_mut(to_dirid) {
                    if let Some(ref mut toch) = to_dirent_mut.children {
                        toch.insert(fileid);
                    }
                }
            }
        }
        let _ = fsmap.refresh_entry(from_dirid).await;
        if to_dirid != from_dirid {
            let _ = fsmap.refresh_entry(to_dirid).await;
        }

        Ok(())
    }
    async fn mkdir(
        &self,
        dirid: fileid3,
        dirname: &filename3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        self.create_fs_object(dirid, dirname, &CreateFSObject::Directory)
            .await
    }

    async fn symlink(
        &self,
        dirid: fileid3,
        linkname: &filename3,
        symlink: &nfspath3,
        attr: &sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        self.create_fs_object(
            dirid,
            linkname,
            &CreateFSObject::Symlink((*attr, symlink.clone())),
        )
        .await
    }
    async fn readlink(&self, id: fileid3) -> Result<nfspath3, nfsstat3> {
        let fsmap = self.fsmap.lock().await;
        let ent = fsmap.find_entry(id)?;
        let path = fsmap.sym_to_path(&ent.name).await;
        drop(fsmap);
        if path.is_symlink() {
            if let Ok(target) = path.read_link() {
                Ok(target.as_os_str().as_bytes().into())
            } else {
                Err(nfsstat3::NFS3ERR_IO)
            }
        } else {
            Err(nfsstat3::NFS3ERR_BADTYPE)
        }
    }
}

const HOSTPORT: u32 = 11111;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();

    let path = std::env::args()
        .nth(1)
        .expect("must supply directory to mirror");
    let path = PathBuf::from(path);

    let fs = EternalFS::new(path);
    let listener = NFSTcpListener::bind(&format!("127.0.0.1:{HOSTPORT}"), fs)
        .await
        .unwrap();
    listener.handle_forever().await.unwrap();
}
// Test with
// mount -t nfs -o nolocks,vers=3,tcp,port=12000,mountport=12000,soft 127.0.0.1:/ mnt/
