# MiniRust

We have now fully desugared our Rust program. The resulting program should only use a limited set of
basic operations. At that level, type-checking ideally is really just a check that doesn't change
program behavior.

This is the level at which we can start to talk about precise semantics. The state-of-the art for
this, that exists today, is [MiniRust](https://github.com/minirust/minirust). MiniRust is a tiny
language that resembles MIR, with a formal and executable semantics.

My hope with these desugarings is that we can reach a subset of Rust that is as precise as MiniRust
is today, thus bridging the gap from source Rust to MiniRust. I don't think we're quite there yet
but hopefully this is a good step in that direction!

Thanks for reading, please open issues if you find mistakes or missing details, and let me know[^1] if
you found this useful!

[^1]: I'm @Nadrieril on the [rust-lang Zulip](https://rust-lang.zulipchat.com), that's the easiest way to reach me.
