libtz
=====

High(er) level Rust interface for [libtz-sys](https://github.com/caldwell/libtz-sys).

Links: [[Documentation](https://docs.rs/libtz/latest)]
       [[Git Repository](https://github.com/caldwell/libtz)]
       [[Crates.io](https://crates.io/crates/libtz)]

Usage
-----

Add this to your `Cargo.toml`:

```toml
[dependencies]
libtz = "0.1.0"
```

Example
-------

```rust
use libtz::{Timezone, TimeT};
use std::time::{SystemTime, UNIX_EPOCH};

let tz = libtz::Timezone::default()?;
let tm = tz.localtime(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as TimeT)?;
println!("tm = {:?}", tm);
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

License
-------
Copyright © 2023 David Caldwell <david@porkrind.org>

MIT Licensed. See [LICENSE.md](LICENSE.md) for details.
