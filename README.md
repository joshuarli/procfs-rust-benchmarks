When writing a program that heavily reads from linux procfs, like a `top` clone,
it's important to question how efficient it is.

It turns out [it isn't a very fast interface](https://jrl.ninja/snip/how-fast-is-procfs.html),
mainly due to massive system call overhead involved in reading lots of files, and
the needless processing required to present what could just be binary data as human-readable.

### Question 1: Is a partial read from procfs faster than a complete read?

In other words, does the kernel lazily do more work as more values are requested?
Or will it simply do everything once something attempts to read at all from it?

Rust's `read_to_end` will steadily turn up the read bytes, resulting in multiple read syscalls:

```
openat(AT_FDCWD, "/proc/stat", O_RDONLY|O_CLOEXEC) = 3
fcntl(3, F_GETFD)                       = 0x1 (flags FD_CLOEXEC)
read(3, "cpu  4266430 4370 99526 42196036", 32) = 32
read(3, "6 29110 0 2472 0 0 0\ncpu0 152285", 32) = 32
read(3, "2 2322 50434 211582847 19216 0 1"..., 64) = 64
read(3, "2 210377518 9894 0 1076 0 0 0\nin"..., 128) = 128
read(3, "0 0 37 484207 0 1322 0 0 0 0 0 0"..., 256) = 256
read(3, " 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"..., 512) = 512
mmap(0x7fc0afcd4000, 14336, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_ANONYMOUS, -1, 0) = 0x7fc0afcd4000
close(3)
```

This is an honorable heuristic, but undesirable behavior for our use case.
It's much better to perform one read.
Then we have to guess a good buffer size - 4096 should be adequate for most files in procfs.

But what if the kernel lazily generates stuff? In the case of /proc/stat I'm only interested in
the first line which summarizes CPU time spent.



### Question 2: How expensive is it to read `/proc/[pid]/status` compared to `/proc/[pid]/stat`?

If so, it might make for a good case to write a completely separate memory watching program.
