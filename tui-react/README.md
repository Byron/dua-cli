**tui-react** is a library to enable components with state, and with properties provided per render

Please note that this crate is early in development and build for the needs of **dua**.

### How it works

It uses the TUI infrastructure Terminal, but alters it to not enforce implementing the `Widget` trait. 
It provides only a single, optional, trait called `TopLevelComponent`, which makes it convenient to
draw its implementors with `Terminal::render(..)`. However, since this enforces the absence of
refernces in your state, it's probably not suitable for most.

Instead, any struct can implement `render` methods or functions, and freely write into the terminal.
That way, one can leverage everything Rust has to offer, which allows stateful components which
work in your favor. Thus, this crate does away with 'one size fits all' render implementations,
greatly adding to flexibility.

State that one wants within the component for instance could be the scoll location. Alternatively,
one can configure windows by altering their public state.

### What's the relation to TUI?

This project coudln't exist without it, and is happy to provide an alternative set of components
for use in command-line applications.


### Why `tui-react`?

I kept having a terrible feeling when managing state with tui widgets when writing **dua**, and
after trying many things, I realized what the problem actually was. It took me some time to
realize it's not possible to have stateful components in with TUI, and I admire the smarts
that went into the API design! After all, it effectively prohibited this! Amazing!

That's why I implemented my own terminal and some key components, based on the ones provided
by TUI, which can serve as standard building blocks in a stateful world.

Thus far, the experience was fantastic, and it feels much better than before. Let's see what
happens with it.

### Changelog

#### v0.4.1 - Simplify `block_width(…)` function

#### v0.2.1 - add license file to crate
