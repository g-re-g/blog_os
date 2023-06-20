# GregOS
An exploration in to how Operating Systems work and a cheeky way to display my
resume.

![Boot Screen](https://raw.githubusercontent.com/g-re-g/greg_os/main/images/boot.png "boot screen")

Based heavily on the amazing https://os.phil-opp.com/ .

## Building
You must have [qemu](https://www.qemu.org/) and [rust](https://rustup.rs/) installed.

- clone this repository
- `cd greg_os`
- add the target `rustup target add thumbv7em-none-eabihf`
- add bootimage `cargo install bootimage`
- `cargo run`
