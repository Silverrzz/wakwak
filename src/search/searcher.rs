use crate::engine::EngineOptions;
use crate::position::Position;
use crate::search::tt::TranspositionTable;
use crate::search::{
    History, MAX_PLY, MoveStack, SearchInfo, SearchStack, TimeManager, iterative_deepening,
};
use crate::uci::SearchLimit;
use crate::util::{BatchedAtomicCounter, Receiver, Sender, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};
use std::thread::JoinHandle;

pub struct Searcher {
    pub shared: Arc<SharedData>,
    threads: Vec<JoinHandle<()>>,
    sender: Sender<ThreadCommand>,
}

impl Searcher {
    #[inline]
    pub fn search(
        &mut self,
        position: Position,
        options: EngineOptions,
        limits: Vec<SearchLimit>,
        info: SearchInfo,
    ) {
        assert!(
            !self.is_searching(),
            "Called `Searcher::search()``while searching"
        );

        self.shared.num_searching.store(1, Ordering::Relaxed);
        self.shared
            .time_man
            .init(position.board().stm(), &limits, options);
        self.sender.send(ThreadCommand::Search {
            position,
            options,
            limits,
            info,
        });
    }

    #[inline]
    pub fn set_threads(&mut self, threads: u32) {
        assert!(
            !self.is_searching(),
            "Called `Searcher::set_threads()` while searching"
        );
        assert!(threads >= 1);

        self.sender.send(ThreadCommand::Quit);
        self.threads.drain(..).for_each(|t| t.join().unwrap());
        self.respawn_threads(threads);
    }

    #[inline]
    pub fn resize_tt(&mut self, size_mb: usize) {
        assert!(
            !self.is_searching(),
            "Called `Searcher::resize_tt()` while searching"
        );

        let threads = self.threads.len() as u32;

        self.sender.send(ThreadCommand::Quit);
        self.threads.drain(..).for_each(|t| t.join().unwrap());

        self.shared = Arc::new(SharedData {
            nodes: Arc::new(AtomicU64::new(0)),
            time_man: TimeManager::default(),
            tt: TranspositionTable::new(size_mb),
            num_searching: AtomicU32::new(0),
            best_score: AtomicI32::new(0),
        });

        self.respawn_threads(threads);
    }

    #[inline]
    fn respawn_threads(&mut self, threads: u32) {
        let (tx, rx) = channel(threads);
        self.threads = rx
            .enumerate()
            .map(|(i, rx)| {
                std::thread::spawn({
                    let shared = self.shared.clone();

                    move || {
                        if std::panic::catch_unwind(move || thread_loop(rx, shared, i)).is_err() {
                            std::process::exit(-1);
                        }
                    }
                })
            })
            .collect();

        self.sender = tx;
        self.sender.send(ThreadCommand::Sync);
    }

    #[inline]
    pub fn newgame(&mut self) {
        assert!(
            !self.is_searching(),
            "Called `Searcher::newgame()` while searching"
        );
        self.shared.tt.clear();
        self.sender.send(ThreadCommand::NewGame);
    }

    #[inline]
    pub fn quit(&mut self) {
        self.shared.time_man.set_stop(true);
        self.sender.send(ThreadCommand::Quit);
        self.threads.drain(..).for_each(|t| t.join().unwrap());
    }

    #[inline]
    pub fn stop(&self) {
        assert!(
            self.is_searching(),
            "Called `Searcher::stop()` while not searching"
        );
        self.shared.time_man.set_stop(true);
    }

    #[inline]
    pub fn wait(&self) {
        let mut num_searching = self.shared.num_searching.load(Ordering::Acquire);
        while num_searching != 0 {
            atomic_wait::wait(&self.shared.num_searching, num_searching);
            num_searching = self.shared.num_searching.load(Ordering::Acquire);
        }
    }

    #[inline]
    pub fn is_searching(&self) -> bool {
        self.shared.num_searching.load(Ordering::Relaxed) != 0
    }
}

impl Default for Searcher {
    #[inline]
    fn default() -> Self {
        let shared = Arc::new(SharedData::default());
        let (mut tx, mut rx) = channel(1);
        let thread = std::thread::spawn({
            let shared = shared.clone();

            move || {
                if std::panic::catch_unwind(move || thread_loop(rx.next().unwrap(), shared, 0))
                    .is_err()
                {
                    std::process::exit(-1);
                }
            }
        });
        tx.send(ThreadCommand::Sync);

        Self {
            shared,
            threads: vec![thread],
            sender: tx,
        }
    }
}

fn thread_loop(mut rx: Receiver<ThreadCommand>, shared: Arc<SharedData>, id: usize) {
    let mut thread = ThreadData::new(shared.nodes.clone(), id);

    loop {
        match rx.recv(|cmd| cmd.clone()) {
            ThreadCommand::Search {
                position,
                options,
                limits: _,
                info,
            } => {
                shared.num_searching.fetch_add(1, Ordering::Relaxed);

                thread.reset();
                iterative_deepening(position, &mut thread, &shared, options, info);
            }
            ThreadCommand::NewGame => {
                thread.history = unsafe { Box::new_zeroed().assume_init() };
            }
            ThreadCommand::Sync => {}
            ThreadCommand::Quit => return,
        }
    }
}

pub struct SharedData {
    pub nodes: Arc<AtomicU64>,
    pub time_man: TimeManager,
    pub tt: TranspositionTable,
    pub num_searching: AtomicU32,
    pub best_score: AtomicI32,
}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        Self {
            nodes: Arc::new(AtomicU64::new(0)),
            tt: TranspositionTable::default(),
            time_man: TimeManager::default(),
            num_searching: AtomicU32::new(0),
            best_score: AtomicI32::new(0),
        }
    }
}

pub struct ThreadData {
    pub nodes: BatchedAtomicCounter,
    pub move_stack: MoveStack,
    pub stack: Vec<SearchStack>,
    pub history: Box<History>,
    pub nmr_ply: Option<usize>,
    pub iid_iteration: usize,
    pub sel_depth: usize,
    pub stop: bool,
    pub id: usize,
}

impl ThreadData {
    #[inline]
    pub fn new(nodes: Arc<AtomicU64>, id: usize) -> Self {
        Self {
            nodes: BatchedAtomicCounter::new(nodes),
            move_stack: MoveStack::default(),
            stack: vec![SearchStack::default(); MAX_PLY + 1],
            history: unsafe { Box::new_zeroed().assume_init() },
            nmr_ply: None,
            iid_iteration: 0,
            sel_depth: 0,
            stop: false,
            id,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.nodes.reset();
        self.stack = vec![SearchStack::default(); MAX_PLY + 1];
        self.move_stack.reset();
        self.sel_depth = 0;
        self.stop = false;
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ThreadCommand {
    Search {
        position: Position,
        options: EngineOptions,
        limits: Vec<SearchLimit>,
        info: SearchInfo,
    },
    NewGame,
    Sync,
    Quit,
}
