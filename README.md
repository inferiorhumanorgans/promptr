# promptr

Because what the world needs is another powerline clone.

In true not-invented-here fashion every powerline-esque prompt prettifier I tried wasn't quite there.  The ones written in interpreted languages are *slow*, the handful of ones written in Rust are poorly documented and/or supported.

The goal with `promptr` is to create a fancy prompt tool that has a somewhat narrow scope, sensible defaults, actionable error messages, reasonable documentation, is fast, fail-safe (who wants their prompt to disappear completely?), and easily configurable.

To that end currently the only shell that's supported is [bash](https://www.gnu.org/software/bash/), and only on non-Windows platforms.

Wait.  What's powerline?  A discussion for another time…

## Installation

`promptr` is written in Rust.  Until precompiled binaries are available a full-blown [Rust build environment](https://rustup.rs/) is required.  `promptr` is mostly tested against the nightly toolchain but should compile with stable as well.  No minimum supported Rust version has been determined although it's probably in the 1.40s.

### Features

For those looking to slim things down a bit, the following segments can be disabled by disabling their respective [feature](https://doc.rust-lang.org/cargo/reference/features.html):

* git -> `segment-git`
* battery -> `segment-battery`

### Compilation

From git:

```sh
git clone https://github.com/inferiorhumanorgans/promptr && \
cd promptr && \
cargo install --path $(pwd)
```

If I ever get around to publishing the crate:

```sh
cargo install promptr
```

### Shell Integration

After installing the `promptr` binary, the next step is to tell `bash` to call `promptr` each time a new prompt is rendered.  For your first time this will do the trick:

```bash
source <(promptr init)>
```

`promptr init` will ensure that the configuration directory exists and if no configuration file exists the default configuration will be written to disk.

To persist this across every invocation add that to your `.bash_profile`, `.profile`, or `.bashrc` as appropriate.  Which file your instance of `bash` will load depends on your operating system and local configuration.  If you would like a quieter startup experience place this in your `bash` file instead:

```bash
source <(promptr load)>
```

## Usage

Batteries *are* included.  Once you run the init command you don't need to do anything else.  If you want to add or remove segments, change colors, or change icons the configuration file is in a platform specific location.  Run `promptr location` to get the path to the configuration directory on your system.  For e.g.:

```
$ uname -s
Darwin
$ whoami
crash_override
$ promptr location
/Users/crash_override/Library/Application Support/com.inferiorhumanorgans.promptr
$ file "$(promptr location)/promptr.json"
/Users/crash_override/Library/Application Support/com.inferiorhumanorgans.promptr/promptr.json: ASCII text
```

```
$ uname -s
FreeBSD
$ whoami
beastie
$ promptr location
/home/beastie/.config/promptr
$ file "$(promptr location)/promptr.json"
/home/beastie/.config/promptr/promptr.json: JSON data
```

For more information check out the documentation with:

```sh
cargo doc --no-deps --open
```

This will compile the documentation and open the top-level index in a new browser window.  Drop the `--open` argument if you want to build the documentation without opening a new browser window.

## Fun and games

```sh
# Run jq on some JSON in-place
jqi() {
    if [ "x$1" = "x" ]; then
        echo "Missing arguments"
        return
    fi
    if [ "x$2" = "x" ]; then
        echo "Missing arguments"
        return
    fi
    contents="$(jq "$1" "$2")" && echo -E "${contents}" > "$2"
    unset contents
}

# Change the git symbol to "(git)"
jqi  '.theme.vcs.symbols.git = "(git)"' ~/Library/Application\ Support/com.inferiorhumanorgans.promptr/promptr.json

# Revert back to the default git symbol
jqi 'del(.theme.vcs.symbols.git)' ~/Library/Application\ Support/com.inferiorhumanorgans.promptr/promptr.json

# Set the git symbol to "", don't do this.  Use the appropriate flag instead
jqi  '.theme.vcs.symbols.git = ""' ~/Library/Application\ Support/com.inferiorhumanorgans.promptr/promptr.json
```
