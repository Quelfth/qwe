
An editor inspired by [Neovim](github.com/neovim/neovim) and [Helix](github.com/helix-editor/helix)

# Features

Qwe has four modes:



## Insert Mode

Keybindings such as `i` and `a` will enter insert mode.
In this mode, characters typed will be inserted at each cursor.

You can exit this mode by pressing escape.

## Mirror Insert Mode

Pressing `[`, or `]` will enter Mirror Insert Mode.  In this mode, each
cursor has both a forward cursor and a reverse cursor.  The forward cursor types
the character you type, and the reverse cursor will type the same character's
flipped version.  So for instance, if you type a `(` while in insert mode, the
reverse cursor will type `)` instead.  If you type a character with no reverse,
that character will simply be typed at both cursors.

You can exit this mode by pressing escape.

## Select Mode

This is the default mode.  In this mode, cursors represent range-selections, and keys
will trigger various functions, rather than typing.  See the keybindings section.

## Line Select Mode

This mode can be entered by pressing `;`.
In this mode, selections always apply to ranges of lines rather than ranges of text.
Otherwise it's similar to Select Mode, but some keybindings are different.

## Keybindings

- `ctrl q` quit the editor

- Scrolling
    - Use `ctrl u` / `ctrl d` to scroll up and down, and `ctrl r` / `ctrl a` to scroll right and left.
    - These bindings plus `alt` will scroll 10 lines instead of 4.

- Insert Mode
    - `tab` will insert indentation if there is no text before your cursor, and invoke autocomplete if there is text.
    - `shift tab` will remove indentation.  Backspace when you are just to the left of indentation will remove not only
      the indentation, but also the line itself, so `shift tab` is necessary to remove indentation
    - `ctrl z` to undo.
    - `ctrl v` to paste from the internal clipboard.

- Lsp Actions
  Qwe supports the language server protocol, and will automatically invoke `rust-analyzer`, `clangd`, the Kotlin Lsp,
  and the C# Roslyn lsp (although this one has some bugs I'm still working out)
  The following actions are supported right now:
  - `'` to show the hover information for a symbol
  - `2` to invoke code actions
  - `@` rename symbol
  - `*` goto definition
  - `alt 8` goto declaration
  - `alt *` goto implementation
  - `&` goto references
  - `Y` goto type definition

- Select Mode and Line Select Mode
  - `ctrl o` open file
  - `(` previous file
  - `)` next file
  - `z` undo
  - `Z` redo
  - `ctrl s` save
  - `f3` view log
  - `f6` inspect highlighting
  - `n` open navigator
  - `ctrl c` copy to system clipboard
  - `"` view diagnostic under cursor
  
  - `esc` all non-main cursors disapear
  - `9` previous cursor
  - `0` next cursor
  - `8` scroll to put main cursor on screen
  - `space` jump to word
  - `f` find in selections
  - `F` find in file
  - `ctrl l` jump to line
  - `1` next diagnostic

  - `tab` tab lines in
  - `shift tab` tab lines out
  - `o` incremental select
  - `:` split cursors across lines
  - `u` collapse cursors to their start point
  - `q` collapse cursors to their end point
  - `alt 9` flit backward (main cursor and the cursor before swap, including contents)
  - `alt 0` flit forward (main cursor and the cursor after swap, including contents)
  - `return` use tree-sitter to separate code across lines syntactically
  - `backspace` remove all line-breaks in the selection
  - `X` delete
  - `x` cut
  - `c` copy
  - `v` paste

  The following case conversions are not implemented correctly, and only apply to the first word of an identifier:
  - `6` `camelCase`
  - `^` `PascalCase`
  - `_` `snake_case`
  - `alt ^` `Ada_Case`
  - `alt _` `SCREAMING_SNAKE_CASE`
  - `-` `kebab-case`
  - `alt 6` `Train-Case`
  - `alt -` `COBOL-CASE`

- Select Mode only
  - `i` insert before
  - `a` insert after
  - `I` insert before line
  - `A` insert after line
  - `[` mirror insert inwards
  - `]` mirror insert outwards
  - `w` word extend
  - `;` line select mode
  - `\` shape selection to text
  - `|` shape selection as block
  
  - `h` move left
  - `j` move right
  - `k` move up
  - `l` move right
  
  - `H` move end-point left
  - `J` move end-point right
  - `K` move end-point up
  - `L` move end-point right

  - `alt h` move start-point left
  - `alt j` move start-point right
  - `alt k` move start-point up
  - `alt l` move start-point right

- Line Select Mode only
  - `i` insert on line before
  - `a` insert on line after
  - `I` insert before line
  - `A` insert after line
  - `[` mirror insert around inwards
  - `]` mirror insert around inwards
  - `;` select mode
  - `\` textual select
  - `|` block select

  - `j` move down
  - `k` move up

  - `J` move bottom-point down
  - `K` move bottom-point up

  - `alt j` move top-point down
  - `alt k` move top-point up

- Navigator
  - `j` move down
  - `k` move up
  - `h` move out
  - `l` move in
  - `i` create directory entry within
  - `a` create directory entry next to
  - `@` rename entry
  - `X` delete entry (only if empty)
  - `n` or `return` edit file
