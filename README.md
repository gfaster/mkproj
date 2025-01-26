# mkproj

My tool for setting up new projects, specifically with Nix. I can never
remember how to properly set up a project with a `shell.nix`, so I figured the
best solution is overkill automation.

The languages currently supported are:
- Rust
- Python
- C
- C++

Future planned supported languages are:
- Haskell
- Latex
- Lean 4
- Go


## Wishlist

- specific project type templates, e.g. Vulkan, Wayland
- include licenses
- config file for default preferences, e.g. license, README, Rust channel
- automatically add packages by library name
- support for environments other than Nix
    - Debian
    - Guix?
- custom templates
