<div align="center">

# wakwak
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

| Version | COPE Duck Chess Elo |
| --- | --- |
| 0.16.0 | - |
| 0.15.0 | - |
| 0.14.0 | - |
| 0.13.0 | - |
| 0.12.0 | - |
| 0.11.0 | - |
| 0.10.0 | - |
| 0.9.0 | - |
| 0.8.0 | - |
| 0.7.0 | - |
| 0.6.0 | - |
| 0.5.0 | - |
| 0.4.0 | - |
| 0.3.0 | - |
| 0.2.0 | - |
| 0.1.0 | - |
| 0.0.0 | 0 |

## External stuff

You can...
1. Play against WakWak online at https://play.wakwak.live/.
2. Download a fully fledged out Duck Chess GUI for analysis and engine games at https://github.com/Silverrzz/pond
3. See a rating list of a handful Duck Chess engines at https://cope-chess.live/ratings?rating_list_id=68
4. View all of our released WakWak neural networks at https://github.com/Silverrzz/wakwak-nets/releases
5. Run your own Duck Chess engine games, and SPRT tests in a familiar CLI using our fork of fastchess at https://github.com/Silverrzz/fastchess
6. Process your own Duck Chess datasets using our adaptation of viriformat and pawnocchio tools at https://github.com/Silverrzz/wakformat
7. Train your own Duck Chess NNUE using our fork of bullet at https://github.com/Silverrzz/bullet

## Duck Tech
WakWak has plenty of novel search features that are unique to a Duck Chess engine.
...

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
