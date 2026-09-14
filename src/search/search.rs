use crate::board::TerminalState;
use crate::common::{Move, Square};
use crate::engine::EngineOptions;
use crate::eval::eval;
use crate::position::Position;
use crate::score::Score;
use crate::search::{MovePicker, Params, PrincipalVariation, SearchInfo, SharedData, ThreadData};
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
    let mut pv = PrincipalVariation::default();
    let mut score = None;
    let alpha = -Score::INFINITE;
    let beta = Score::INFINITE;

    'id: loop {
        thread.sel_depth = 0;
        let new_score = Some(search::<Root>(
            &mut pos,
            thread,
            shared,
            alpha,
            beta,
            depth as i32,
            0,
        ));
        thread.nodes.flush();

        if depth > 1 && thread.stop {
            break 'id;
        }

        score = new_score;
        pv = thread.stack[0].pv.clone();

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

    // Wait for `stop` command if search is infinite
    if shared.time_man.infinite() {
        shared.time_man.wait_for_stop();
    }

    let last_thread = shared.num_searching.fetch_sub(1, Ordering::AcqRel) == 2;

    // The last thread to decrement wakes the main thread, unless the last thread is the main thread.
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
            pv[0].display(options.dumb_interface, options.frc)
        );
    }

    // Wake the other threads after printing
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

    parent.pv.clear();
    parent.pv.push(mv);
    parent.pv.extend(child.pv.iter().copied());
}

fn search<Node: NodeType>(
    pos: &mut Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    mut alpha: Score,
    beta: Score,
    depth: i32,
    ply: usize,
) -> Score {
    if !Node::ROOT && (thread.stop || shared.time_man.stop_search(thread)) {
        shared.time_man.set_stop(true);
        thread.stop = true;

        return Score::ZERO;
    }

    if Node::PV {
        thread.stack[ply].pv.clear();
    }

    thread.sel_depth = thread.sel_depth.max(ply);

    // TODO: node counting has to be changed once qsearch is implemented
    if !Node::ROOT {
        thread.nodes.inc();
    }

    if let Some(terminal_state) = pos.board().terminal_state() {
        return match terminal_state {
            TerminalState::Victory(_) => Score::mated(ply),
            TerminalState::Stalemate(_) => Score::mate(ply),
            TerminalState::Draw => Score::draw(),
        };
    }

    if !Node::ROOT && pos.repetition() {
        return Score::draw();
    }

    let static_eval = eval(pos.board());

    if depth <= 0 {
        return static_eval;
    }

    if !Node::ROOT
        && depth <= Params::rfp_depth()
        && static_eval - Params::rfp_margin(depth) >= beta
    {
        return static_eval;
    }

    let mut best_move = None;
    let mut best_score = None;

    thread.move_stack.push(pos.board());
    let mut failed_quiets = Vec::new();
    let mut failed_noisies = Vec::new();
    let mut move_picker = MovePicker::default();
    let mut move_count = 0;
    let mut duck_counts: [[u8; Square::COUNT]; Square::COUNT] = [[0; Square::COUNT]; Square::COUNT];

    while let Some(mv) = move_picker.next(pos, thread) {
        let (src, dest) = (mv.src(), mv.dest());
        let is_quiet = mv.flag().is_quiet();
        let count = &mut duck_counts[src][dest];

        if is_quiet && depth <= Params::ldp_depth() && *count >= Params::ldp_threshold(depth) as u8
        {
            continue;
        }

        *count += 1;
        pos.make_move(mv);
        let score = -search::<PV>(pos, thread, shared, -beta, -alpha, depth - 1, ply + 1);
        pos.unmake_move();

        if Node::ROOT && move_count == 0 {
            update_pv(thread, mv, ply);
        }

        if thread.stop {
            thread.move_stack.pop();
            return Score::ZERO;
        }

        move_count += 1;

        if score > best_score {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            if Node::PV {
                update_pv(thread, mv, ply);
            }

            if score >= beta {
                thread.history.update(
                    pos.board(),
                    depth,
                    best_move.unwrap(),
                    &failed_quiets,
                    &failed_noisies,
                );
                break;
            }
        }

        if best_move != Some(mv) {
            if mv.flag().is_noisy() {
                failed_noisies.push(mv);
            } else {
                failed_quiets.push(mv);
            }
        }
    }

    thread.move_stack.pop();
    best_score.unwrap()
}
