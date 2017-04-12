var searchIndex = {};
searchIndex["fifo"] = {"doc":"A first-in-first-out bytes ring-buffer like kfifo in Linux.","items":[[3,"Sender","fifo","The fifo sender. It's `Send` but `!Send`.",null,null],[3,"Receiver","","The fifo receiver. It's `Send` but `!Send`.",null,null],[4,"WouldBlock","","What can we do when operations on `Sender` or `Receiver` would block.",null,null],[13,"Nonblock","","",0,null],[13,"Sleep","","",0,null],[5,"fifo","","Construct the fifo with capacity as `size.next_power_of_two()`, and return the `Sender` and `Receiver` pair connected with that.",null,null],[0,"splice","","Zero-copy interface for `Sender` and `Receiver`, in splice style in Linux. When you want to write bytes into the ring-buffer from a `Read`, or read bytes from this into a `Write`, you can use `SpliceRead` and `SpliceWrite`.",null,null],[8,"SpliceRead","fifo::splice","Copy some bytes directly from another `Read` object, without use any temporary buffers.",null,null],[10,"splice_from","","Copy at most bytes bytes from a `Read` r.",1,{"inputs":[{"name":"self"},{"name":"t"},{"name":"usize"}],"output":{"name":"result"}}],[11,"splice_all_from","","Copy all bytes from a `Read` r.",1,{"inputs":[{"name":"self"},{"name":"t"}],"output":{"name":"result"}}],[8,"SpliceWrite","","Copy some bytes directly into another `Write` object, without use any temporary buffers.",null,null],[10,"splice_to","","Copy at most bytes bytes into a `Write` w.",2,{"inputs":[{"name":"self"},{"name":"t"},{"name":"usize"}],"output":{"name":"result"}}],[11,"splice_all_to","","Copy all bytes to a `Write` w.",2,{"inputs":[{"name":"self"},{"name":"t"}],"output":{"name":"result"}}],[11,"splice_from","fifo","",3,{"inputs":[{"name":"self"},{"name":"t"},{"name":"usize"}],"output":{"name":"result"}}],[11,"splice_to","","",4,{"inputs":[{"name":"self"},{"name":"t"},{"name":"usize"}],"output":{"name":"result"}}],[11,"as_ref","","",3,{"inputs":[{"name":"self"}],"output":{"name":"inner"}}],[11,"as_ref","","",4,{"inputs":[{"name":"self"}],"output":{"name":"inner"}}],[11,"drop","","",3,{"inputs":[{"name":"self"}],"output":null}],[11,"drop","","",4,{"inputs":[{"name":"self"}],"output":null}],[11,"set_would_block","","",3,{"inputs":[{"name":"self"},{"name":"wouldblock"}],"output":null}],[11,"write","","",3,null],[11,"flush","","",3,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"set_would_block","","",4,{"inputs":[{"name":"self"},{"name":"wouldblock"}],"output":null}],[11,"read","","",4,null]],"paths":[[4,"WouldBlock"],[8,"SpliceRead"],[8,"SpliceWrite"],[3,"Sender"],[3,"Receiver"]]};
initSearch(searchIndex);
