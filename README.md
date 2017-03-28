# fifo
Fifo is a first-in-first-out bytes ring-buffer, like [kfifo][kfifo in Linux] in Linux.

**API documentation**
* [master](http://hicqu.github.io/fifo)

## Features

* `Sender` and `Receiver` implements `Write` and `Read`, like `mpsc::channel`.
* [splice][man splice] style zero-copy when write into and read from the ring-buffer.
* lock-free concurrent access between one producer and one consumer.

[kfifo in Linux]: http://lxr.free-electrons.com/source/include/linux/kfifo.h
[man splice]: https://linux.die.net/man/2/splice

## Usage
To use `fifo`, first add this to your `Cargo.toml`:
```toml
fifo = "0.1.*"
```

Then, add this to your crate root:
```rust
extern crate fifo;
```

## Examples
Here is an [example](https://github.com/hicqu/fifo-rs/tree/master/examples).

## Contributing

For simple bug fixes, just submit a PR with the fix and we can discuss
the fix directly in the PR. If the fix is more complex, start with an issue.

If you want to propose an API change, create an issue to start a
discussion with the community. Also, feel free to talk with us in the
IRC channel.
