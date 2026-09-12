use crate::board::TerminalState;
use crate::common::Move;
use crate::engine::EngineOptions;
use crate::position::Position;
use crate::score::Score;
use crate::search::{PrincipalVariation, SearchInfo};
use crate::search::{SharedData, ThreadData};
use rand::RngExt;
use rand::rngs::ThreadRng;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Default)]
pub struct SearchStack {
    pv: PrincipalVariation,
}

pub fn iterative_deepening(
    mut pos: Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    options: EngineOptions,
    info: SearchInfo,
) {
    let mut depth = 1;
    let mut completed_depth = 0;
    let mut score;
    let mut pv;

    let mut rng = rand::rng();

    'id: loop {
        score = Some(search::<Root>(
            &mut pos,
            thread,
            shared,
            &mut rng,
            depth as i32,
            0,
        ));
        pv = thread.stack[0].pv.clone();
        thread.nodes.flush();

        if depth > 1 && thread.stop {
            break 'id;
        }

        if thread.id == 0 {
            if shared.time_man.stop_id(depth, thread.nodes.global()) {
                shared.time_man.set_stop(true);
                thread.stop = true;
                break 'id;
            }

            shared.time_man.deepen(depth);
        }

        depth += 1;
        completed_depth += 1;

        if thread.id == 0 && info == SearchInfo::Full {
            info.depth(
                thread,
                shared,
                options,
                completed_depth,
                score.unwrap(),
                &pv,
            );
        }
    }

    //Wait for `stop` command if search is infinite
    if shared.time_man.infinite() {
        shared.time_man.wait_for_stop();
    }

    let last_thread = shared.num_searching.fetch_sub(1, Ordering::AcqRel) == 2;

    //The last thread to decrement wakes the main thread, unless the last thread is the main thread.
    if last_thread && thread.id != 0 {
        atomic_wait::wake_all(&shared.num_searching);
    }

    if thread.id == 0 {
        // The main thread ensures all search threads have finished before printing
        if !last_thread {
            let mut num_searching = shared.num_searching.load(Ordering::Acquire);
            while num_searching != 1 {
                atomic_wait::wait(&shared.num_searching, num_searching);
                num_searching = shared.num_searching.load(Ordering::Acquire);
            }
        }

        // All search threads have finished, we are ready for new commands.
        shared.num_searching.store(0, Ordering::Release);
    }

    if thread.id == 0 && info != SearchInfo::None {
        info.depth(
            thread,
            shared,
            options,
            completed_depth,
            score.unwrap(),
            &pv,
        );
        println!(
            "bestmove {}",
            pv.moves[0]
                .unwrap()
                .display(options.dumb_interface, options.frc)
        );
    }

    //Wake the other threads after printing
    if thread.id == 0 {
        atomic_wait::wake_all(&shared.num_searching);
    }
}

trait NodeType {
    const PV: bool;
    const ROOT: bool;
}

struct Root;
struct PV;
#[expect(dead_code)]
struct NonPV;

impl NodeType for Root {
    const PV: bool = true;
    const ROOT: bool = true;
}

impl NodeType for PV {
    const PV: bool = true;
    const ROOT: bool = false;
}

impl NodeType for NonPV {
    const PV: bool = false;
    const ROOT: bool = false;
}

#[inline]
fn update_pv(thread: &mut ThreadData, mv: Move, ply: usize) {
    let [parent, child] = thread.stack.get_disjoint_mut([ply, ply + 1]).unwrap();

    parent.pv.update(mv, &child.pv);
}

fn search<Node: NodeType>(
    pos: &mut Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    rng: &mut ThreadRng,
    depth: i32,
    ply: usize,
) -> Score {
    if !Node::ROOT && (thread.stop || shared.time_man.stop_search(thread)) {
        shared.time_man.set_stop(true);
        thread.stop = true;

        return Score::ZERO;
    }

    if Node::PV {
        thread.stack[ply].pv.len = 0;
    }

    thread.sel_depth = thread.sel_depth.max(ply);
    if let Some(terminal_state) = pos.board().terminal_state() {
        thread.nodes.inc();
        return match terminal_state {
            TerminalState::Victory(_) => Score::mated(ply),
            TerminalState::Stalemate(_) => Score::mate(ply),
            TerminalState::Draw => Score::draw(),
        };
    }

    //TODO: Threefold Repetition, make a function `Position::repetition()` or something

    if !Node::ROOT {
        thread.nodes.inc();
    }

    if depth <= 0 {
        return Score(rng.random_range((-Score::MAX_MATE.0 + 1)..=(Score::MAX_MATE.0 - 1)));
    }

    //FIXME: Remove leading _ when this is used
    let mut _best_move = None;
    let mut best_score = None;

    let moves = pos.board().gen_moves();
    for (move_count, &mv) in moves.iter().enumerate() {
        pos.make_move(mv);
        let score = -search::<PV>(pos, thread, shared, rng, depth - 1, ply + 1);
        pos.unmake_move();

        if Node::ROOT && move_count == 0 {
            update_pv(thread, mv, ply);
        }

        if thread.stop {
            return Score::ZERO;
        }

        //TODO: Alpha-Beta Pruning will require moving stuff like PV updates
        if score > best_score {
            best_score = Some(score);
            _best_move = Some(mv);

            if Node::PV {
                update_pv(thread, mv, ply);
            }
        }
    }

    best_score.unwrap()
}
