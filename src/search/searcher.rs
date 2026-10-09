use crate::common::Move;
use crate::engine::EngineOptions;
use crate::position::Position;
use crate::score::Score;
use crate::search::tt::TranspositionTable;
use crate::search::{
    Bound, History, MAX_PLY, MoveStack, PrincipalVariation, SearchInfo, SearchStack, TimeManager,
    iterative_deepening,
};
use crate::uci::SearchLimit;
use crate::util::{Abort, BatchedAtomicCounter, Receiver, Sender, channel};
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
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
        self.shared.init(&position, &limits, options);
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

        self.shared = Arc::new(SharedData::new(size_mb));

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

                {
                    let root_moves = shared.root_moves.read().unwrap();

                    thread.root_moves.clear();
                    thread.root_moves.reserve(root_moves.len());
                    thread.root_moves.extend_from_slice(&root_moves);

                    thread.multipv = options.multipv.min(root_moves.len());
                }

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

#[derive(Clone, Debug)]
pub struct RootMove {
    pub score: Score,
    pub window_score: Score,
    pub display_score: Score,
    pub previous_score: Score,
    pub bound: Bound,
    pub searched_depth: usize,
    pub sel_depth: usize,
    pub pv: PrincipalVariation,
}

impl RootMove {
    pub fn new(mv: Move) -> Self {
        let mut result = Self {
            score: -Score::INFINITE,
            window_score: -Score::INFINITE,
            display_score: -Score::INFINITE,
            previous_score: -Score::INFINITE,
            bound: Bound::None,
            searched_depth: 1,
            sel_depth: 0,
            pv: Default::default(),
        };
        result.pv.push(mv);
        result
    }
}

pub struct SharedData {
    pub nodes: Arc<AtomicU64>,
    pub time_man: TimeManager,
    pub tt: TranspositionTable,
    pub num_searching: AtomicU32,
    pub root_moves: RwLock<Vec<RootMove>>,
    pub best_score: AtomicI32,
}

impl SharedData {
    fn new(tt_size_mb: usize) -> Self {
        Self {
            nodes: Arc::new(AtomicU64::new(0)),
            time_man: TimeManager::default(),
            tt: TranspositionTable::new(tt_size_mb),
            num_searching: AtomicU32::new(0),
            root_moves: RwLock::new(Vec::new()),
            best_score: AtomicI32::new(0),
        }
    }

    fn init(&self, position: &Position, limits: &[SearchLimit], options: EngineOptions) {
        self.time_man.init(position.board().stm(), &limits, options);
        let mut root_moves = self.root_moves.write().unwrap();
        root_moves.clear();
        position.board().gen_all_moves(|moves| {
            moves
                .iter()
                .map(RootMove::new)
                .for_each(|root_move| root_moves.push(root_move));
            Abort::No
        });
    }
}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        Self {
            nodes: Arc::new(AtomicU64::new(0)),
            tt: TranspositionTable::default(),
            time_man: TimeManager::default(),
            num_searching: AtomicU32::new(0),
            root_moves: RwLock::new(Vec::new()),
            best_score: AtomicI32::new(0),
        }
    }
}

pub struct ThreadData {
    pub root_moves: Vec<RootMove>,
    pub nodes: BatchedAtomicCounter,
    pub move_stack: MoveStack,
    pub stack: Vec<SearchStack>,
    pub history: Box<History>,
    pub nmr_ply: Option<usize>,
    pub iid_iteration: usize,
    pub root_depth: usize,
    pub sel_depth: usize,
    pub multipv: usize,
    pub pv_idx: usize,
    pub stop: bool,
    pub id: usize,
}

impl ThreadData {
    #[inline]
    pub fn new(nodes: Arc<AtomicU64>, id: usize) -> Self {
        Self {
            root_moves: Vec::new(),
            nodes: BatchedAtomicCounter::new(nodes),
            move_stack: MoveStack::default(),
            stack: vec![SearchStack::default(); MAX_PLY + 1],
            history: unsafe { Box::new_zeroed().assume_init() },
            nmr_ply: None,
            iid_iteration: 0,
            root_depth: 0,
            sel_depth: 0,
            multipv: 1,
            pv_idx: 0,
            stop: false,
            id,
        }
    }

    pub fn get_root_move_idx(&self, mv: Move) -> usize {
        self.root_moves
            .iter()
            .position(|root_move| root_move.pv[0] == mv)
            .unwrap()
    }

    pub fn is_legal_root_move(&self, mv: Move) -> bool {
        self.root_moves[self.pv_idx..]
            .iter()
            .any(|root_move| root_move.pv[0] == mv)
    }

    pub fn pv_move(&self) -> &RootMove {
        &self.root_moves[0]
    }

    pub fn sort_searched_root_moves(&mut self) {
        self.root_moves[..=self.pv_idx].sort_by_key(|root_move| std::cmp::Reverse(root_move.score));
    }

    pub fn sort_remaining_root_moves(&mut self) {
        self.root_moves[self.pv_idx..].sort_by_key(|root_move| std::cmp::Reverse(root_move.score));
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
