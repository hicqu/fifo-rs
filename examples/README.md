# fifo-examples
It's a collection of some simple examples of [fifo][fifo].
Now, it just contains only one use case.

# how it works
There are two threads when the program is running. One thread read a file
named *rust.avi* in current directory and write content into a `Sender`.
The another one reads bytes from the `Receiver` respectively, then write
them into a disk file named *tsur.avi*.

After the program exits, `diff rust.avi tsur.avi` should print nothing
which means these two files are exactly same. So we can believe the `Sender`
and `Receiver` work correctly.

[fifo]: (http://hicqu.github.io/fifo-rs)
