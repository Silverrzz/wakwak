use crate::board::Board;
use crate::common::{Move, MoveFlag, Square};
use crate::engine::{Engine, EngineOptions};
use crate::nnue::Nnue;
use crate::search::SearchInfo;
use crate::uci::SearchLimit;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::sync::atomic::Ordering;
use std::time::Instant;
use wakformat::common::MoveFlag as WfMoveFlag;
use wakformat::format::{self, FILE_HEADER, GAME_HEADER_SIZE, MoveRecord, RECORD_SIZE};

impl Engine {
    pub fn relabel(
        &mut self,
        input: &str,
        output: &str,
        nodes: u64,
        threads: usize,
    ) -> io::Result<()> {
        let mut reader = BufReader::with_capacity(4 * 1024 * 1024, File::open(input)?);
        format::read_file_header(&mut reader)?;
        let mut writer = BufWriter::with_capacity(4 * 1024 * 1024, File::create_new(output)?);
        writer.write_all(&FILE_HEADER)?;
        let options = EngineOptions {
            soft_target: true,
            contempt: 0,
            multipv: 1,
            ..self.options
        };
        let start = Instant::now();
        let mut positions = 0;
        loop {
            let mut games = Vec::new();
            let mut size = 0;
            while size < 64 * 1024 * 1024 {
                let mut game = Vec::new();
                if !format::read_game(&mut reader, &mut game)? {
                    break;
                }
                size += game.len();
                games.push(game);
            }
            if games.is_empty() {
                break;
            }
            let chunk_size = games.len().div_ceil(threads);
            positions += std::thread::scope(|scope| {
                let handles: Vec<_> = games
                    .chunks_mut(chunk_size)
                    .map(|chunk| scope.spawn(move || relabel(chunk, nodes, options)))
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .sum::<io::Result<u64>>()
            })?;
            for game in games {
                writer.write_all(&game)?;
            }
            eprintln!(
                "Relabeled {positions} positions in {:.1}s",
                start.elapsed().as_secs_f64()
            );
        }
        writer.flush()
    }
}

fn relabel(games: &mut [Vec<u8>], nodes: u64, options: EngineOptions) -> io::Result<u64> {
    let mut nnue = Nnue::new(&Board::startpos());
    let mut search = (nodes != 0).then(Engine::new);
    let result = (|| {
        let mut positions = 0;
        for game in games {
            let mut packed =
                format::unpack_header((&game[..GAME_HEADER_SIZE]).try_into().unwrap())?;
            let mut board = Board::from_fen(&packed.to_fen(true))
                .ok_or_else(|| format::invalid("invalid initial position"))?;
            nnue.full_reset(&board);
            if let Some(engine) = &mut search {
                engine.position.reset(board);
            }
            let end = game.len() - RECORD_SIZE;
            for bytes in game[GAME_HEADER_SIZE..end].as_chunks_mut::<RECORD_SIZE>().0 {
                let record = MoveRecord::from_bytes(*bytes)
                    .map_err(format::invalid)?
                    .ok_or_else(|| format::invalid("unexpected game terminator"))?;
                format::replay_move(&mut packed, record.mv)?;
                let mv = Move::new(
                    Square::index(record.mv.src() as usize),
                    Square::index(record.mv.dest() as usize),
                    Square::index(record.mv.duck() as usize),
                    match record.mv.flag() {
                        WfMoveFlag::Normal => MoveFlag::Normal,
                        WfMoveFlag::DoublePush => MoveFlag::DoublePush,
                        WfMoveFlag::LongCastling => MoveFlag::LongCastling,
                        WfMoveFlag::ShortCastling => MoveFlag::ShortCastling,
                        WfMoveFlag::PromotionQueen => MoveFlag::PromotionQueen,
                        WfMoveFlag::PromotionRook => MoveFlag::PromotionRook,
                        WfMoveFlag::PromotionBishop => MoveFlag::PromotionBishop,
                        WfMoveFlag::PromotionKnight => MoveFlag::PromotionKnight,
                        WfMoveFlag::Capture => MoveFlag::Capture,
                        WfMoveFlag::EnPassant => MoveFlag::EnPassant,
                        WfMoveFlag::CapturePromotionQueen => MoveFlag::CapturePromotionQueen,
                        WfMoveFlag::CapturePromotionRook => MoveFlag::CapturePromotionRook,
                        WfMoveFlag::CapturePromotionBishop => MoveFlag::CapturePromotionBishop,
                        WfMoveFlag::CapturePromotionKnight => MoveFlag::CapturePromotionKnight,
                    },
                );
                let score = if let Some(engine) = &mut search {
                    engine.position.reset_nnue();
                    engine.searcher.search(
                        engine.position.clone(),
                        options,
                        vec![SearchLimit::Nodes(nodes)],
                        SearchInfo::None,
                    );
                    engine.searcher.wait();
                    engine.searcher.shared.best_score.load(Ordering::Relaxed)
                } else {
                    nnue.update(&board);
                    nnue.stack.swap(0, nnue.cursor);
                    nnue.cursor = 0;
                    nnue.eval(&board).0
                };
                let score = (score * i32::from(board.stm().signum()))
                    .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                bytes[3..].copy_from_slice(&score.to_le_bytes());
                if let Some(engine) = &mut search {
                    engine.position.make_move(mv);
                } else {
                    nnue.make_move(&board, mv);
                }
                board.make_move(mv);
            }
            positions += ((game.len() - GAME_HEADER_SIZE) / RECORD_SIZE - 1) as u64;
        }
        Ok(positions)
    })();
    if let Some(engine) = &mut search {
        engine.searcher.quit();
    }
    result
}
