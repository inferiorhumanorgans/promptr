## Notes on Testing `git(1)`

Archives are created with GNU tar via:

```
gtar -vc --owner=0 --group=0 --no-same-owner --no-same-permissions -f 
```

* `cherry-pick` covers the "interactive" case
* `empty` covers the unborn branch case (new repo)
* `rebase-interactive` covers the interactive rebase case
* `untracked-file` covers the untracked files in the repo case
