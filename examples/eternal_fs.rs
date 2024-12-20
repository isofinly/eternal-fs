use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::io::SeekFrom;
use std::ops::Bound;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

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
use rand::Rng;

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
    Beginning,
    Logic,      // New: Logic puzzles and rationality
    Emotion,    // New: Emotional exploration
    Identity,   // New: Self-discovery
    Time,       // New: Temporal mechanics
    Creation,   // New: Creative forces
    History,    // New: Past reflections
    Myth,       // New: Mythological understanding
    Perception, // New: Reality questioning
    Quantum,    // New: Uncertainty principles
    Chaos,      // New: Unpredictability
    Enlightened,
}

impl GameStage {
    fn next(&self) -> Option<GameStage> {
        match self {
            GameStage::Beginning => Some(GameStage::Logic),
            GameStage::Logic => Some(GameStage::Emotion),
            GameStage::Emotion => Some(GameStage::Identity),
            GameStage::Identity => Some(GameStage::Time),
            GameStage::Time => Some(GameStage::Creation),
            GameStage::Creation => Some(GameStage::History),
            GameStage::History => Some(GameStage::Myth),
            GameStage::Myth => Some(GameStage::Perception),
            GameStage::Perception => Some(GameStage::Quantum),
            GameStage::Quantum => Some(GameStage::Chaos),
            GameStage::Chaos => Some(GameStage::Enlightened),
            GameStage::Enlightened => None,
        }
    }
}

#[derive(Debug, Clone)]
struct PhilosophicalState {
    emotional_state: String,
    perception_filters: HashSet<String>,
    quantum_states: HashMap<String, bool>,
    created_elements: Vec<String>,
    timeline_events: Vec<(SystemTime, String)>,
    solved_puzzles: HashSet<String>,
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
    philosophical_state: PhilosophicalState,
    rng: Arc<Mutex<StdRng>>,
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
            philosophical_state: PhilosophicalState {
                emotional_state: "neutral".to_string(),
                perception_filters: HashSet::new(),
                quantum_states: HashMap::new(),
                created_elements: Vec::new(),
                timeline_events: Vec::new(),
                solved_puzzles: HashSet::new(),
            },
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        };

        map.initialize_game_world();
        map
    }

    fn initialize_game_world(&mut self) {
        // Create root with introduction
        let root_entry = FSEntry {
            name: Vec::new(),
            fsmeta: metadata_to_fattr3(1, &self.root.metadata().unwrap()),
            children_meta: metadata_to_fattr3(1, &self.root.metadata().unwrap()),
            children: None,
            philosophical_content: Some(PhilosophicalContent {
                question: "Welcome to the Philosophical Filesystem. What truth do you seek?"
                    .to_string(),
                responses: Vec::new(),
                last_interaction: SystemTime::now(),
            }),
        };

        self.id_to_path.insert(0, root_entry);
        self.path_to_id.insert(Vec::new(), 0);

        // Create all philosophical directories with their questions
        let directories = vec![
            ("logic", "If this statement is false, what is truth?"),
            ("emotion", "Can an emotion exist without being felt?"),
            (
                "identity",
                "If you change every part of yourself, are you still you?",
            ),
            (
                "time",
                "Does the present moment truly exist between past and future?",
            ),
            ("creation", "Can something come from nothing?"),
            ("history", "How do past choices shape current reality?"),
            ("myth", "What eternal truths lie within stories?"),
            ("perception", "Is your reality the only reality?"),
            (
                "quantum",
                "Can something exist in multiple states until observed?",
            ),
            ("chaos", "Is there order in randomness?"),
        ];

        for (name, question) in directories {
            self.create_philosophical_directory(name, question);
        }

        // Create special files
        self.create_quantum_state_file();
        self.create_perception_filter();
        self.create_timeline_tracker();

        // Initialize progress file
        self.update_progress_file();
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
        let response_quality = response.len() > 50;

        let (reply, should_advance) = match (location, &self.current_stage, response_quality) {
            // Logic Path
            ("logic", GameStage::Beginning, true)
                if response.contains("paradox") && response.contains("truth") =>
            {
                self.completed_questions.insert("logic".to_string());
                (
                    "The paradox dissolves as you grasp its essence. Truth is both the question and the answer.".to_string(),
                    true
                )
            }
            // Emotion Path
            ("emotion", GameStage::Logic, true) if response.contains("feel") => {
                self.completed_questions.insert("emotion".to_string());
                (
                    "Your emotional awareness creates ripples in the fabric of reality."
                        .to_string(),
                    true,
                )
            }
            // Identity Path
            ("identity", GameStage::Emotion, true)
                if response.contains("change") && response.contains("constant") =>
            {
                self.completed_questions.insert("identity".to_string());
                (
                    "You understand that identity persists through change, like a river always flowing."
                        .to_string(),
                    true,
                )
            }
            // Time Path
            ("time", GameStage::Identity, true)
                if response.contains("present") && response.contains("future") =>
            {
                self.completed_questions.insert("time".to_string());
                (
                    "Time reveals itself as both infinite and instantaneous. The moment contains eternity."
                        .to_string(),
                    true,
                )
            }
            // Creation Path
            ("creation", GameStage::Time, true)
                if response.contains("create") && response.contains("existence") =>
            {
                self.completed_questions.insert("creation".to_string());
                (
                    "Through creation, you understand the nature of existence itself.".to_string(),
                    true,
                )
            }
            // History Path
            ("history", GameStage::Creation, true)
                if response.contains("past") && response.contains("memory") =>
            {
                self.completed_questions.insert("history".to_string());
                (
                    "The patterns of history reveal themselves in your understanding.".to_string(),
                    true,
                )
            }
            // Myth Path
            ("myth", GameStage::History, true)
                if response.contains("story") && response.contains("truth") =>
            {
                self.completed_questions.insert("myth".to_string());
                (
                    "The eternal truths hidden in stories become clear to you.".to_string(),
                    true,
                )
            }
            // Perception Path
            ("perception", GameStage::Myth, true)
                if response.contains("reality") && response.contains("illusion") =>
            {
                self.completed_questions.insert("perception".to_string());
                (
                    "Your perception shifts, revealing the many layers of reality.".to_string(),
                    true,
                )
            }
            // Quantum Path
            ("quantum", GameStage::Perception, true)
                if response.contains("uncertainty") && response.contains("possibility") =>
            {
                self.completed_questions.insert("quantum".to_string());
                (
                    "You grasp the quantum nature of reality through its inherent uncertainty."
                        .to_string(),
                    true,
                )
            }
            // Chaos Path
            ("chaos", GameStage::Quantum, true)
                if response.contains("order") && response.contains("chaos") =>
            {
                self.completed_questions.insert("chaos".to_string());
                (
                    "In the heart of chaos, you discover the deepest order.".to_string(),
                    true,
                )
            }
            // Enlightenment Path (Final Stage)
            (_, GameStage::Chaos, true)
                if response.contains("understanding") && response.contains("wisdom") =>
            {
                self.completed_questions.insert("enlightenment".to_string());
                (
                    "You have reached enlightenment. All paths converge in understanding."
                        .to_string(),
                    true,
                )
            }
            // Response too short
            (_, _, false) => (
                format!(
                    "Your response must be more thoughtful (>50 characters). Current length: {}",
                    response.len()
                ),
                false,
            ),
            // Wrong stage or location
            _ => (
                format!(
                    "You are currently in the {:?} stage. The path of {} is not yet ready for you.",
                    self.current_stage, location
                ),
                false,
            ),
        };

        // Advance stage if needed
        if should_advance {
            if let Some(next_stage) = self.current_stage.next() {
                self.current_stage = next_stage;
                self.update_progress_file();
            }
        }

        reply
    }

    fn update_progress_file(&mut self) {
        let mut progress_path = self.root.clone();
        progress_path.push("progress.txt");
        let progress_content = format!(
            "Journey Progress\n\
            ===============\n\n\
            Current Stage: {:?}\n\
            Progress: {}/11\n\n\
            Active Challenge: {}\n\
            Next Stage: {}\n\n\
            Hint: {}\n",
            self.current_stage,
            self.completed_questions.len(),
            self.get_current_challenge(),
            self.get_next_stage_name(),
            self.get_current_hint()
        );
        let _ = std::fs::write(progress_path, progress_content);
    }

    fn get_current_challenge(&self) -> String {
        match self.current_stage {
            GameStage::Beginning => "Understand the nature of truth and paradox".to_string(),
            GameStage::Logic => "Experience and understand pure emotions".to_string(),
            GameStage::Emotion => "Contemplate the nature of identity".to_string(),
            GameStage::Identity => "Reflect on the nature of time".to_string(),
            GameStage::Time => "Create something meaningful".to_string(),
            GameStage::Creation => "Reflect on your past choices".to_string(),
            GameStage::History => "Decode the myths that shape your beliefs".to_string(),
            GameStage::Myth => "Examine your perception of reality".to_string(),
            GameStage::Perception => "Explore the uncertainties of quantum mechanics".to_string(),
            GameStage::Quantum => "Find order in chaos".to_string(),
            GameStage::Chaos => "Achieve enlightenment through understanding".to_string(),
            GameStage::Enlightened => "You have completed all challenges".to_string(),
        }
    }

    fn get_next_stage_name(&self) -> String {
        match self.current_stage {
            GameStage::Beginning => "Logic".to_string(),
            GameStage::Logic => "Emotion".to_string(),
            GameStage::Emotion => "Identity".to_string(),
            GameStage::Identity => "Time".to_string(),
            GameStage::Time => "Creation".to_string(),
            GameStage::Creation => "History".to_string(),
            GameStage::History => "Myth".to_string(),
            GameStage::Myth => "Perception".to_string(),
            GameStage::Perception => "Quantum".to_string(),
            GameStage::Quantum => "Chaos".to_string(),
            GameStage::Chaos => "Enlightenment".to_string(),
            GameStage::Enlightened => "Complete".to_string(),
        }
    }

    fn get_current_hint(&self) -> String {
        match self.current_stage {
            GameStage::Beginning => {
                "Consider: Can truth contain its own contradiction?".to_string()
            }
            GameStage::Logic => "Feel deeply and express your emotional understanding".to_string(),
            GameStage::Emotion => "Reflect on what makes you who you are".to_string(),
            GameStage::Identity => "What remains when everything changes?".to_string(),
            GameStage::Time => "Is the present moment truly real?".to_string(),
            GameStage::Creation => "Can something come from nothing?".to_string(),
            GameStage::History => "How do past choices shape your current reality?".to_string(),
            GameStage::Myth => "What stories shape your understanding of the world?".to_string(),
            GameStage::Perception => "How do you know what you perceive is real?".to_string(),
            GameStage::Quantum => "What changes when you observe it?".to_string(),
            GameStage::Chaos => "What patterns do you see in randomness?".to_string(),
            GameStage::Enlightened => "Reflect on your journey".to_string(),
        }
    }

    fn create_special_file(&mut self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let mut file_path = self.root.clone();
        file_path.push(filename);

        // Create the file with content
        std::fs::write(&file_path, content)?;

        // Create virtual filesystem entry
        if let Ok(meta) = file_path.metadata() {
            let file_sym = self.intern.intern(OsString::from(filename)).unwrap();
            let file_name = vec![file_sym];
            let file_id = self.next_fileid.fetch_add(1, Ordering::Relaxed);

            let file_entry = FSEntry {
                name: file_name.clone(),
                fsmeta: metadata_to_fattr3(file_id, &meta),
                children_meta: metadata_to_fattr3(file_id, &meta),
                children: None,
                philosophical_content: None,
            };

            // Add to mappings
            self.id_to_path.insert(file_id, file_entry);
            self.path_to_id.insert(file_name, file_id);
        }

        Ok(())
    }

    fn create_quantum_state_file(&mut self) {
        let content = "\
            Quantum State Observation Log\n\
            ==========================\n\
            This file exists in a superposition of states.\n\
            Each read may collapse it into a different reality.\n\
            \n\
            Current State: [SUPERPOSITION]\n\
            Probability Field: Active\n\
            Observer Effect: Enabled\
        ";

        let _ = self.create_special_file("quantum_state.txt", content);
    }

    fn create_perception_filter(&mut self) {
        let content = "\
            Perception Filters\n\
            =================\n\
            Your perception shapes the reality of this filesystem.\n\
            \n\
            Active Filters:\n\
            - Default Reality\n\
            \n\
            Available Filters:\n\
            - Truth Lens\n\
            - Quantum Vision\n\
            - Temporal Sight\
        ";

        let _ = self.create_special_file("perception.txt", content);
    }

    fn create_timeline_tracker(&mut self) {
        let content = "\
            Timeline Tracker\n\
            ===============\n\
            Past, present, and future converge in this space.\n\
            \n\
            Current Timeline: Alpha\n\
            Temporal Stability: 100%\n\
            \n\
            Recent Events:\n\
            - Timeline initialized\n\
            - Quantum fluctuations detected\n\
            - Reality matrix stable\
        ";

        let _ = self.create_special_file("timeline.txt", content);
    }

    // Add helper method to update special files
    async fn update_special_file(&mut self, filename: &str, new_content: &str) {
        let mut file_path = self.root.clone();
        file_path.push(filename);
        let _ = tokio::fs::write(&file_path, new_content).await;
    }

    // Add method to update quantum state randomly
    async fn update_quantum_state(&mut self) {
        let state = {
            let mut rng = self.rng.lock().await;
            if rng.gen_bool(0.5) {
                "COLLAPSED: PARTICLE"
            } else {
                "COLLAPSED: WAVE"
            }
        };

        let content = format!(
            "\
            Quantum State Observation Log\n\
            ==========================\n\
            State collapsed by observation.\n\
            \n\
            Current State: [{}]\n\
            Last Observation: {:?}\n\
            Coherence: {:.2}%\
        ",
            state,
            SystemTime::now(),
            {
                let mut rng = self.rng.lock().await;
                rng.gen_range(0.0..100.0)
            }
        );

        self.update_special_file("quantum_state.txt", &content)
            .await;
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

        // Handle special files first
        if let Some(filename) = path.file_name() {
            match filename.to_str() {
                Some("quantum_state.txt") => {
                    fsmap.update_quantum_state().await;
                    // Early return as quantum state is randomly generated
                    return Ok(metadata_to_fattr3(id, &path.metadata().unwrap()));
                }
                Some("answer.txt") => {
                    if let Ok(content) = String::from_utf8(data.to_vec()) {
                        let location = path
                            .parent()
                            .map(|p| p.strip_prefix(&fsmap.root).unwrap_or(p))
                            .and_then(|p| p.to_str())
                            .unwrap_or("");

                        let response = fsmap
                            .process_philosophical_response(location, &content)
                            .await;

                        // Create system_response.txt in the same directory
                        let mut response_path = path.clone();
                        response_path.set_file_name("system_response.txt");
                        tokio::fs::write(&response_path, response).await.ok();
                    }
                }
                _ => {}
            }
        }

        // Continue with normal write operation
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
// mount -t nfs -o nolocks,vers=3,tcp,port=12000,mountport=12000,soft 127.0.0.1:/ eternal
