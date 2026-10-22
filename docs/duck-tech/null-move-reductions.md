**Null move reductions** by 87flowers\
https://github.com/Silverrzz/wakwak/pull/71\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L374-L409

Null move pruning was attempted and failed. We assume that this is because of the prevalance of zugzwang, especially duckzwang.
We instead attempt a large reduction if a null move search fails high.
A successful further null move search within a reduced subtree results in a prune.
