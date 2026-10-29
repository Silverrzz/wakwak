use super::config::{Config, Format};
use super::source::{board, san};
use super::{Record, Result};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

pub fn write(config: &Config, records: &[Record]) -> Result<()> {
    let path = &config.output.path;
    std::fs::create_dir_all(path.parent().ok_or("output needs a parent directory")?)?;
    let file = OpenOptions::new().create_new(true).write(true).open(path)?;
    let result = (|| -> Result<()> {
        let mut output = BufWriter::new(file);
        for (index, record) in records.iter().enumerate() {
            match config.output.format {
                Format::Fen => writeln!(output, "{}", record.board.to_fen(config.output.chess960))?,
                Format::Epd => {
                    let fen = record.board.to_fen(config.output.chess960);
                    let fields: Vec<_> = fen.split_whitespace().collect();
                    writeln!(
                        output,
                        "{} hmvc {}; fmvn {}; id \"{}\";",
                        fields[..4].join(" "),
                        fields[4],
                        fields[5],
                        index + 1
                    )?;
                }
                Format::Pgn => {
                    let mut position = board(&record.initial_fen)?;
                    let variant = if config.output.chess960 {
                        "Duck960"
                    } else {
                        "Duck"
                    };
                    writeln!(
                        output,
                        "[Event \"Generated book\"]\n[Round \"{}\"]\n[Variant \"{variant}\"]\n[SetUp \"1\"]\n[FEN \"{}\"]\n[Result \"*\"]\n",
                        index + 1,
                        position.to_fen(config.output.chess960)
                    )?;
                    for (ply, &mv) in record.moves.iter().enumerate() {
                        if position.stm() == crate::common::Color::White {
                            write!(output, "{}. ", position.fmc())?;
                        } else if ply == 0 {
                            write!(output, "{}... ", position.fmc())?;
                        }
                        write!(output, "{} ", san(&position, mv))?;
                        position.make_move(mv);
                    }
                    writeln!(output, "*\n")?;
                }
            }
        }
        output.flush()?;
        output.get_ref().sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}
