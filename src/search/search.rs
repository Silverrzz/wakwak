use crate::common::{Bitboard, Move, Square, between};
use crate::engine::EngineOptions;
use crate::eval::eval;
use crate::position::Position;
use crate::score::Score;
use crate::search::cont::ContIndices;
use crate::search::tt::TTFlag;
use crate::search::{MovePicker, Params, PrincipalVariation, SearchInfo, SharedData, ThreadData};
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Default)]
pub struct SearchStack {
    pv: PrincipalVariation,
    raw_eval: Option<Score>,
    static_eval: Option<Score>,
    mv: Option<Move>,
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
fn adjust_eval(eval: Score, corr: i32) -> Score {
    (eval + corr).clamp_mate()
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
    thread.stack[ply].mv = None;

    thread.sel_depth = thread.sel_depth.max(ply);

    // TODO: node counting has to be changed once qsearch is implemented
    if !Node::ROOT {
        thread.nodes.inc();
    }

    // King captured, gg
    if pos.board().try_king(pos.board().stm()).is_none() {
        return Score::mated(ply);
    }

    // 50-move-rule detection
    if pos.board().hmc() >= 100 {
        return Score::draw();
    }

    // Three-fold repetition detection
    if !Node::ROOT && pos.repetition() {
        return Score::draw();
    }

    /*
    Transposition Table Cutoffs (TT Cutoffs): If we've already searched this position
    and the stored result indicates that its value is outside the window, we can return
    that stored result instead of wasting time searching it again.
    */
    let tt_entry = shared.tt.probe(pos.board().hash());
    let mut tt_move = tt_entry.and_then(|e| e.best_move());

    if !Node::ROOT
        && let Some(entry) = tt_entry
    {
        let score = entry.score();
        if entry.depth() >= depth && entry.flag().bounds_match(score, alpha, beta) {
            return score;
        }
    }

    if depth > 0
        && (!Node::PV || tt_move.is_none())
        && let Some(entry) = shared.tt.probe(pos.board().duckless_hash())
        && entry.flag() == TTFlag::Lower
        && !entry.score().is_mate()
    {
        let cutoff = !Node::PV && entry.depth() >= depth && entry.score() >= beta;
        if (cutoff || tt_move.is_none())
            && let Some(mv) = entry.best_move()
            && pos.board().is_legal(mv)
        {
            if cutoff {
                thread.stack[ply].mv = Some(mv);
                return entry.score();
            }
            tt_move = Some(mv);
        }
    }

    // TODO: uncomment this when it is used
    // let in_check = pos.board().in_check();
    let raw_eval = eval(pos.board());
    let corr = thread.history.corr(pos.board());
    let static_eval = adjust_eval(raw_eval, corr);
    let raw_eval = eval(pos.board());

    if depth <= 0 {
        return static_eval;
    }

    let improving = {
        let prev2 = ply.wrapping_sub(2);
        let prev4 = ply.wrapping_sub(4);

        if ply >= 2 && thread.stack[prev2].static_eval.is_some() {
            static_eval > thread.stack[prev2].static_eval
        } else if ply >= 4 && thread.stack[prev4].static_eval.is_some() {
            static_eval > thread.stack[prev4].static_eval
        } else {
            true
        }
    };

    thread.stack[ply].raw_eval = Some(raw_eval);
    thread.stack[ply].static_eval = Some(static_eval);

    /*
    Reverse Futility Pruning: If our evaluation of the position is already
    so high that even a pessimistic estimate is still above beta, we can
    be reasonably confident that a further search will also fail high.
    */
    if !Node::PV
        && depth <= Params::rfp_depth()
        && static_eval - Params::rfp_margin(depth, improving) >= beta
    {
        return static_eval;
    }

    thread.move_stack.push_ply();

    let mut best_move = None;
    let mut best_move_depth = depth;
    let mut best_score = None;
    let mut legal_moves = 0;
    let mut searched_moves = 0;
    let mut failed_quiets = Vec::new();
    let mut failed_noisies = Vec::new();
    let mut move_picker = MovePicker::new(tt_move);
    let mut duck_counts: [[u8; Square::COUNT]; Square::COUNT] = [[0; Square::COUNT]; Square::COUNT];
    let mut duck_refutations = [(None, Bitboard::EMPTY); Square::COUNT];
    let mut duck_safety = [(None, Bitboard::FULL); Square::COUNT];
    let mut flag = TTFlag::Upper;

    let indices = ContIndices::new(pos);
    while let Some(mv) = move_picker.next(pos, thread, indices) {
        let (src, dest) = (mv.src(), mv.dest());
        let piece_move = Some((src, mv.flag()));
        let is_quiet = mv.flag().is_quiet();
        legal_moves += 1;

        /*
        Duck Refutations: If the opponent immediately refutes a duck move,
        we can skip the rest of the duck moves that don't block the refutation(s).
        */
        if duck_refutations[dest].0 == piece_move && duck_refutations[dest].1.has(mv.duck()) {
            continue;
        }

        if duck_safety[dest].0 != Some(src) {
            let mut board = *pos.board();
            // TODO: Calculate king capture blocks without making the full move.
            board.make_move(mv);
            duck_safety[dest] = (Some(src), board.king_capture_blocks(!board.stm()));
        }
        let safe = duck_safety[dest].1;

        /*
        Late Duck Pruning (LDP): After a certain number of duck moves for
        a certain move, we can be reasonably confident they're not gonna get
        much better, so we can skip the rest of them.
        */
        if safe == Bitboard::FULL
            && depth <= Params::ldp_depth(is_quiet)
            && duck_counts[src][dest] >= Params::ldp_threshold(depth, is_quiet, improving) as u8
        {
            continue;
        }

        duck_counts[src][dest] += 1;
        pos.make_move(mv);

        /*
        Duck or Die Pruning: Treat duck moves that let the opponent capture
        the king as instant losses, unless it is a repetition.
        */
        let mut move_depth = depth;
        let score = if !safe.has(mv.duck()) && pos.board().hmc() < 100 && !pos.repetition() {
            // Clear the previous child's continuation because this move skips recursive search.
            thread.stack[ply + 1].pv.clear();
            Score::mated(ply + 2)
        } else {
            let new_depth = depth - 1;
            let mut score = -Score::INFINITE;
            if !Node::PV || legal_moves > 1 {
                let reduction = if depth >= 3 && searched_moves > 6 && is_quiet {
                    1
                } else {
                    0
                };
                move_depth -= reduction;
                score = -search::<NonPV>(
                    pos,
                    thread,
                    shared,
                    -alpha - 1,
                    -alpha,
                    new_depth - reduction,
                    ply + 1,
                )
            }
            if Node::PV && (legal_moves == 1 || score > alpha) {
                move_depth = depth;
                score = -search::<PV>(pos, thread, shared, -beta, -alpha, new_depth, ply + 1);
            }
            score
        };
        pos.unmake_move();

        if Node::ROOT && searched_moves == 0 {
            update_pv(thread, mv, ply);
        }

        if thread.stop {
            thread.move_stack.pop_ply();
            return Score::ZERO;
        }

        // Duck Refutations
        if let Some(reply) = thread.stack[ply + 1].mv {
            let refuted = !(between(reply.src(), reply.dest()) | reply.dest() | reply.duck());
            if duck_refutations[dest].0 == piece_move {
                duck_refutations[dest].1 |= refuted;
            } else {
                duck_refutations[dest] = (piece_move, refuted);
            }
        }

        searched_moves += 1;

        if score > best_score {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            best_move_depth = move_depth;
            thread.stack[ply].mv = best_move;
            flag = TTFlag::Exact;
            if Node::PV {
                update_pv(thread, mv, ply);
            }

            if score >= beta {
                flag = TTFlag::Lower;
                thread.history.update(
                    pos.board(),
                    indices,
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

    thread.move_stack.pop_ply();

    // Stalemate detection
    if legal_moves == 0 {
        return Score::mate(ply);
    }

    let best_score = best_score.unwrap();

    if pos.board().duck().is_some()
        && best_move.is_some()
        && matches!(flag, TTFlag::Exact | TTFlag::Lower)
        && !best_score.is_mate()
    {
        shared.tt.insert(
            pos.board().duckless_hash(),
            best_move,
            best_score,
            best_move_depth,
            TTFlag::Lower,
        );
    }

    shared.tt.insert(
        pos.board().hash(),
        best_move,
        best_score,
        best_move_depth,
        flag,
    );

    let static_eval = adjust_eval(raw_eval, thread.history.corr(pos.board()));
    if best_move.is_none_or(|mv| mv.flag().is_quiet())
        && flag.bounds_match(best_score, static_eval, static_eval)
    {
        thread
            .history
            .update_corr(pos.board(), depth, best_score, static_eval);
    }

    best_score
}
