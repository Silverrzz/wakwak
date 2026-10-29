# Duck Tech

- [Duck history, including quiet duck history](duck-tech/duck-history.md)
- [Duck refutations](duck-tech/duck-refutations.md)
- [Duck count, late duck and unique duck pruning](duck-tech/duck-pruning.md)
- [Unique move pruning and reductions](duck-tech/unique-moves.md)
- [Neutral duck pruning and ordering](duck-tech/neutral-ducks.md)
- [King safety: Duck or Die, capture blocks and historical duck shield eval](duck-tech/king-safety.md)
- [Duck-aware static exchange evaluation](duck-tech/duck-aware-see.md)
- [Duckless hashing](duck-tech/duckless-hashing.md)
- [Null move reductions](duck-tech/null-move-reductions.md)
- [Duck stability time management](duck-tech/duck-stability-time-management.md)

# Known techniques

- Negamax alpha-beta search
- Iterative deepening
- Principal variation search
- Aspiration windows
- Quiescence search
- Transposition tables
- TT move ordering
- Staged move generation
- Incremental move sorting
- MVV capture ordering
- Quiet history
- Threat history
- Capture history
- Pawn history
- 1, 2 and 4-ply continuation history
- Pawn, minor, major and non-pawn correction history
- 1 and 2-ply continuation correction history
- History gravity updates
- Futility pruning
- Reverse futility pruning
- Razoring
- Quiet history pruning
- Quiet and noisy SEE pruning
- Late move reductions
- History-adjusted reductions and pruning margins
- Singular extensions
- Double singular extensions
- Hindsight extensions
- Internal iterative deepening
- TT-adjusted static evaluation
- Improving detection
- Lazy SMP
- Move stability time management
- Magic bitboards
- Zobrist hashing
- Cuckoo-based upcoming repetition detection

# NNUE features

WakWak's NNUE is trained on over three and a half billion self-generated Duck Chess positions. Our released networks are available in [wakwak-nets](https://github.com/Silverrzz/wakwak-nets)
and our self-gen data at [wakwak-data](https://huggingface.co/datasets/EllieSilverrzz/wakwak-data).

- 768 piece-square inputs
- 64 duck-square inputs
- 11 bucketed half-move-clock inputs
- Horizontal mirroring
- 1536 neuron layer 1, followed by a 16 and 32 neuron multilayer setup
- L2 skip-connection
- Pairwise multiplication
- Dual activation
- Lazy incremental accumulators
- SIMD inference supporting avx512, avx2, NEON and web assembly
- Permuted for sparsity and dpbusd

## Duck inputs

Duck inputs add 64 features, one for each possible duck square, with the same perspective
and horizontal mirroring as the piece-square inputs.

We previously tried duck inputs with hidden layers of 16, 64, 256 and 768 neurons.
Every attempt lost roughly 30 Elo. Though since [anadama](https://github.com/Silverrzz/wakwak-nets/releases/tag/anadama) our
net's have included them. With enough high-quality training data and a large enough network,
we now see results that show the NNUE can finally make useful evaluations of the duck's placement.

Our (somewhat internally debated :p) hypothesis is that the duck has little value as a
standalone evaluation heuristic. Its placement matters through its interactions with the
pieces and the moves it allows or prevents. A sufficiently capable NNUE can learn those
interactions and interpret what effect the duck will have on search.
