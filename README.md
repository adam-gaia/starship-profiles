# starship-profiles

Wrapper program to add multi-profile support to [starship prompt](https://starship.rs/)

## TL;DR

This is a wrapper that forces `starship` to load different configs depending on the current working directory.
The wrapper reads ~/.config/starship/profiles.toml, and uses the rules there to pick a starship config from ~/.config/starship/profiles/\*.toml, then invokes `starship`, forcing it to use the configuration file.

## Motivation

I want a different prompt configuration depending on the directory I'm in.
Take this for example: The git status module is integral to my starship prompt.
The downside is that starship runs `git status` is ran each time my prompt reloads, which can take a noticeable amount of time.

Here are the timings in this repo:

```console
> starship timings                                                                                                                                                                                                                   31-Mon 09:13:44 AM
 Here are the timings of modules in your prompt (>=1ms or output):
 rust        -  20ms  -   "via  v1.83.0 "
 git_status  -   4ms  -   "[!]"
 directory   -   1ms  -   "~/r/p/starship-profiles "
 character   -  <1ms  -   "> "
 git_branch  -  <1ms  -   "main"
 hostname    -  <1ms  -   "helix"
 line_break  -  <1ms  -   "\n"
 nix_shell   -  <1ms  -   "❄ nix-shell-env "
 package     -  <1ms  -   "is  v0.1.0 "
 username    -  <1ms  -   "agaia"
```

The prompt's delay while running git status is unnodicable. Furthermore, the 20ms it takes to run the rust module is also unnoticeable.

Here is the same prompt, in the largest repo I work with, [nixpkgs](https://github.com/NixOS/nixpkgs):

```console
> starship timings                                                                                                                                                                                                                   31-Mon 09:55:20 AM
 Here are the timings of modules in your prompt (>=1ms or output):
 git_status  -  225ms  -   "[⇡]"
 character   -   <1ms  -   "> "
 directory   -   <1ms  -   "~/r/p/nixpkgs "
 git_branch  -   <1ms  -   "add-rusty-rain"
 hostname    -   <1ms  -   "helix"
 line_break  -   <1ms  -   "\n"
 username    -   <1ms  -   "agaia"
```

Using starship while working in this directory is super annoying. It would be great if I could toggle starship's git-related modules off while working in certain directories, so I built this wrapper.

## Usage

1. Create a '~/.config/starship/profiles.toml', that defines a list of profiles. Each profile contains a name and a list of patterns that trigger the profile.
1. Invoke the wrapper provided in this repo, `starship-profiles`, in place of `starship`.

Take this profiles.toml for example:

```toml
[[profile]]
name = "no-git"
patterns = ["~/repo/nixpkgs", "~/repo/big-project"]

[[profile]]
name = "simple"
patterns = ["~/repo/(.+)-scripts"]
```

This corresponds to two starship configuration "profiles", `~/.config/starship/profiles/no-git.toml` and `~/.config/starship/profiles/simple.toml`.
These profiles use [starship's configuration format](https://starship.rs/config/).

When working in `~/repo/nixpkgs` (or one of its subdirectories), the no-git profile will be used.
Likewise, the no-git profile will activate under `~/repo/big-project`.
A different profile, 'simple' will activate in any directory ending with '-scripts'.
Starship will read the standard config file everywhere else.

Some misc notes:

- The profiles are checked in the order they are listed in the profiles.toml file.
- `~` will get evaluated to the contents of `$HOME`, then that expanded string will be used as the regex
- The name of each starship configuration file must exactly match the profile.name entry in the profiles.toml. Spaces and special characters aren't expressly forbidden, but I haven't tested that sort of thing
- Full regex patterns are supported, but I've found in practice writing out full paths is best (like in the 'no-git' example patterns).
- Regex matching does not require a full match, so subdirectories will also trigger their parents' profiles

## How it works

When the current directory matches a regex, that profile is triggered and starship is invoked by the wrapper with `STARSHIP_CONFIG=~/.config/starship/profiles/<profile-name>.toml starship <args>` to force the specific config to be used.
Should no patterns match, the wrapper will call starship without forcing a config, thus letting starship use its standard config automatically (`~/.config/starship.toml`) or default if no config found.

## Alternatives

- [direnv](https://direnv.net/)
  Direnv allows for setting env vars when entering a directory, then unsetting when leaving.

  ```bash
  #.envrc
  export STARSHIP="${HOME}/.config/starship/profiles/no-git.toml"
  ```

  The downside is that I often git commit my .envrc files and don't want to (or can't) force my changes on other contributors.

- shell hooks
  Shell hooks can be simpler, but are obviously tied to the shell. The advantage to starship-profiles is that it is shell agnostic. starship-profiles makes it easier to add multiple rules

  - zsh
    ```zsh
    set-starship() {
     if [[ "$PWD" =~ ^/home/agaia/repo/big-project ]] || [[ "$PWD" =~ ^/home/agaia/repo/nixpkgs ]]; then
       export STARSHIP_CONFIG="$HOME/.config/starship/profiles/no-git.toml"
     elif [[ "$PWD" =~ ^/home/agaia/repo/small- ]]; then
       export STARSHIP_CONFIG="$HOME/.config/starship/profiles/simple.toml"
     else
       unset STARSHIP_CONFIG
     fi
    }
    add-zsh-hook precmd set-starship
    ```
  - nushell
    TODO
