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

After staring at some perf graphs (`perf record -g ./bench-stat-partial-read`, `perf report --stdio -s cpu` by the way),
the two didn't really look much different. A huge chunk of the time under kernel `show_stat` is indeed
spent in human formatting (`seq_put_decimal_ull` -> `num_to_str`, `seq_printf`)

So, I just decided to benchmark things.

100k loops of complete reads took around ~2.1s, and partial reads of 128 bytes ~1.2s.

Although I had a suspicion it was probably just the 6x read syscalls.
100k loops of doing a single 4096 byte read also ~1.2s.

So in conclusion, no, the kernel does not appear to lazily generate at least the contents of /proc/stat.
And, procfs in general should be consumed with one read syscall, not multiple.
(There are still 2 more syscalls involved - open and close - which is why `readfile` will be useful here.)

The only Rust method I found suitable for doing a single, oneshot read was:

```rust
let mut f = File::open("/proc/stat")?;
let mut buf = [0; 4096];
f.read(&mut buf).unwrap();
```

There is also `read_exact`, although, it actually results in 2 reads. Consider the scenario:

Generated `/proc/stat` will be 1400 bytes.
`read_exact` into a 4096 bytes buffer will send read with 4096, see that it read 1400,
and try read 4096 again. Seeing that ret was 0, it exits.

Because you can't guess the size of `/proc/stat` ahead of time,
(Most files in procfs, when statted, will return st_size 0. Also, that's another syscall.)
and because we can reasonably guess the size (perhaps scale with #cpus), it's fine to do a oneshot read.


### Question 2: How expensive is it to read `/proc/[PID]/status` compared to `/proc/[PID]/statm`?

`/proc/[PID]/status` contains a `VmSwap`, whereas statm doesn't have that information.

Also from the htop FAQ:

> Why doesn't htop feature a SWAP column, like top?

> It is not possible to get the exact size of used swap space of a process. Top fakes this information by making SWAP = VIRT - RES, but that is not a good metric, because other stuff such as video memory counts on VIRT as well (for example: top says my X process is using 81M of swap, but it also reports my system as a whole is using only 2M of swap. Therefore, I will not add a similar Swap column to htop because I don't know a reliable way to get this information (actually, I don't think it's possible to get an exact number, because of shared pages). 

So, I'm not sure how valuable `VmSwap` would actually be.

`/proc/[PID]/status` contains a bunch of other information as well, and it's more humanized.
If it isn't that much slower than reading statm however, I might think more about `VmSwap`.

100k loops of status is ~0.74s, 100k loops of statm is ~0.3s. Clear winner there.
