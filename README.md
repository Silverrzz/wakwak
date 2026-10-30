<div align="center">

# WakWak
<img width="592" height="565" src="https://github.com/user-attachments/assets/d57ef9dc-d4e6-4514-b326-472b50a9de9d">

[![License][license-badge]][license-link]
[![Release][release-badge]][release-link]
[![Commits][commits-badge]][commits-link]

![Rust][rust-badge]
![LGBTQ+ Friendly][lgbtqp-badge]
![Trans Rights][trans-rights-badge]

</div>

The strongest Duck Chess engine in the world.
It supports both standard and [(Double) Fischer Random][dfrc] duck chess,
and is exclusively trained on self-generated training data.

## Strength

<table align="left">
<thead>
<tr><th width="260">Version</th><th width="140"><a href="https://cope-chess.live/ratings?rating_list_id=68">COPE Elo</a></th></tr>
</thead>
<tbody>
<tr><td>1.0.0</td><td>5,331</td></tr>
<tr><td>0.16.0</td><td>5,089</td></tr>
<tr><td>0.15.0</td><td>4,857</td></tr>
<tr><td>0.14.0</td><td>4,711</td></tr>
<tr><td>0.13.0</td><td>4,361</td></tr>
<tr><td>0.12.0</td><td>4,027</td></tr>
<tr><td>0.11.0</td><td>3,794</td></tr>
<tr><td>0.10.0</td><td>3,675</td></tr>
<tr><td>0.9.0</td><td>3,494</td></tr>
<tr><td>0.8.0</td><td>3,377</td></tr>
<tr><td>0.7.0</td><td>3,071</td></tr>
<tr><td>0.6.0</td><td>2,904</td></tr>
<tr><td>0.5.0</td><td>2,748</td></tr>
<tr><td>0.4.0</td><td>2,448</td></tr>
<tr><td>0.3.0</td><td>2,158</td></tr>
<tr><td>0.2.0</td><td>1,904</td></tr>
<tr><td>0.1.0</td><td>1,526</td></tr>
<tr><td>0.0.0</td><td>0</td></tr>
</tbody>
</table>

<table align="center">
<thead>
<tr><th width="260">Engine</th><th width="140"><a href="https://cope-chess.live/ratings?rating_list_id=68">COPE Elo</a></th></tr>
</thead>
<tbody>
<tr><td><strong>wakwak 1.0.0</strong></td><td><strong>5,331</strong></td></tr>
<tr><td>Fairy-Stockfish</td><td>3,429</td></tr>
<tr><td>Duckfish 1.0-SNAPSHOT</td><td>2,741</td></tr>
<tr><td>MayhemDuck 2.4</td><td>2,480</td></tr>
<tr><td>deep-duck 0.2.0</td><td>2,423</td></tr>
<tr><td>DuckChess.jl</td><td>2,312</td></tr>
<tr><td>DuckChessZero 0.1.0</td><td>2,010</td></tr>
<tr><td>Peter Duck 0.1.0</td><td>1,754</td></tr>
<tr><td>Harel Duck</td><td>1,492</td></tr>
<tr><td>Duck Chess Agent</td><td>1,180</td></tr>
<tr><td>Duck Chess RL</td><td>1,008</td></tr>
</tbody>
</table>

<br clear="both">

## UCI Options

| Name             | Type    | Default | Valid Values      | Description                                                                                        |
|------------------|---------|---------|-------------------|----------------------------------------------------------------------------------------------------|
| Threads          | Integer | 1       | `1..=1024`        | Number of search threads                                                                           |
| Hash             | Integer | 64      | `1..=16777216`    | Size of the Transposition Table (in MiB)                                                           |
| MoveOverhead     | Integer | 50      | `0..=5000`        | Time (in ms) used to compensate for the delay between engine and interface communication           |
| Minimal          | Boolean | `false` | `true` or `false` | When enabled, WakWak outputs only the final info line and best move                                |
| SoftTarget       | Boolean | `false` | `true` or `false` | When enabled, `go nodes <n>` and `go movetime <ms>` can only stop after a completed depth          |
| UseDumbInterface | Boolean | `true`  | `true` or `false` | When enabled, WakWak outputs moves in the format used by Fairy Stockfish                           |
| MultiPV          | Integer | 1       | `1..=16384`      | Number of principal variations to search and report                                               |
| Strength         | Integer | 20      | `-20..=20`      | Playing strength                       |
| Contempt         | Integer | 0       | `0..=200`        | Bias against draws; higher values make WakWak prefer continuing the game                           |
| UCI_Chess960     | Boolean | `false` | `true` or `false` | Whether to output moves using standard notation (e1g1/e1c1) or Chess960 notation (e.g. e1h1, e1a1) |
| UCI_ShowWDL      | Boolean | `false` | `true` or `false` | Include estimated win, draw and loss probabilities in the search output                            |
| ScoreNormalisation | Boolean | `true` | `true` or `false` | Normalise reported centipawn scores using the WDL model.     |

## Go Parameters

| Parameter | Description |
|-----------|-------------|
| `wtime <ms>`, `btime <ms>` | Remaining time for White and Black |
| `winc <ms>`, `binc <ms>` | Increment per move for White and Black |
| `movetime <ms>` | Time limit for this move |
| `nodes <n>` | Node limit for the search |
| `depth <n>` | Maximum search depth |
| `infinite` | Search until `stop` is received |
| `searchmoves <moves...>` | Only search the listed root moves or wildcard matches |
| `excludemoves <moves...>` | Exclude the listed root moves or wildcard matches |

## Building
If your device can't run the latest release of WakWak, you want to use the latest development version,
or you otherwise want to compile WakWak yourself, you can build it from source.

WakWak requires Make, Python 3 and any version of rustup.
The required toolchain and version will be automatically installed.

```bash
> make EXE=<NAME>
```
- Replace `<NAME>` with the desired name for the binary. The default name is `WakWak`.
- `EVALFILE=<FILE>` can also be passed in to build a binary with a specific neural network embedded, though the code must be changed to reflect this network's architecture.

Since neural networks are extremely large files, WakWak's neural networks are stored in
[a separate repository][wakwak-nets] to avoid bloating this repository's size. Because of this,
you need to run `make` **at least once** before you can use `cargo` to build and run WakWak.

## External stuff

You can...
1. Play against WakWak online at https://play.wakwak.live/.
2. Play against WakWak online at https://www.pychess.org/@/wakwak
3. Download our Duck Chess GUI for analysis and engine games at https://github.com/Silverrzz/pond
4. See a rating list of a handful Duck Chess engines at https://cope-chess.live/ratings?rating_list_id=68
5. View all of our released WakWak neural networks at https://github.com/Silverrzz/wakwak-nets/releases
6. Run your own Duck Chess engine games, and SPRT tests in a familiar CLI using our fork of fastchess at https://github.com/Silverrzz/fastchess
7. Process your own Duck Chess datasets using our adaptation of viriformat and pawnocchio tools at https://github.com/Silverrzz/wakformat
8. Train your own Duck Chess NNUE using our fork of bullet at https://github.com/Silverrzz/bullet
9. Use our generated Duck Chess opening books at https://github.com/Silverrzz/wakwak-books, or generate your own.

## Features
WakWak combines novel Duck Chess search features with a multilayer NNUE.
Read about our [Duck Tech](docs/README.md#duck-tech), [known techniques](docs/README.md#known-techniques), and [NNUE features](docs/README.md#nnue-features).

## Contributing
Code contributions are welcome to WakWak! Go ahead and fork this repository, and make your patch on a decently well named branch. Test the change using a sufficient SPRT setup, and make a PR to this repository.

Precommit hooks are provided to ensure the code is properly formatted and free of warnings. To enable them, install [pre-commit](https://pre-commit.com/) and run `pre-commit install`. If you only want the hooks to run before pushing, run `pre-commit install -t pre-push` instead. To install a hook that ensures every commit message contains a valid bench, additionally run `pre-commit install -t commit-msg`.

We also welcome hardware contributions to our OpenBench instance, MattBench. You can reach out to us via Discord at @silverrzz to find out how to do this.

[license-badge]: https://img.shields.io/github/license/Silverrzz/wakwak?style=for-the-badge
[release-badge]: https://img.shields.io/github/v/release/Silverrzz/wakwak?style=for-the-badge
[commits-badge]: https://img.shields.io/github/commits-since/Silverrzz/wakwak/latest?style=for-the-badge

[license-link]: https://github.com/Silverrzz/wakwak/blob/main/LICENSE
[release-link]: https://github.com/Silverrzz/wakwak/releases/latest
[commits-link]: https://github.com/Silverrzz/wakwak/commits/main

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&color=red
[lgbtqp-badge]: https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982
[trans-rights-badge]: https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA

[dfrc]: https://en.wikipedia.org/wiki/Chess960
[wakwak-nets]: https://github.com/Silverrzz/wakwak-nets
