# Hammer

An option parsing library that deserializes flags into structs.

```rust
#![feature(phase)]

extern crate serialize;
#[phase(plugin, link)]
extern crate hammer;

use std::os;
use hammer::{decode_args, usage};

#[deriving(Decodable, Show)]
struct MyOpts {
    string : Option<String>,
    verbose : bool,
    rest : Vec<String> // any extra flags
}

hammer_config!(MyOpts "A test of hammer.rs", // note the description line
    |c| { c
        .short("verbose", 'v') // short versions of variables
        // .rest_field("remaining") // if you want to put the "extra"
                                    // arguments in a different field
    }
)

fn main() {
    let opts: MyOpts = decode_args(os::args().tail()).unwrap();
    println!("opts given: {}", opts);
    
    let (desc, usage_text) = usage::<MyOpts>(true);
    println!("Usage: {}", os::args().get(0));
    println!("{}", usage_text);
    println!("{}", desc.unwrap())
}
```

Several different types are allowed within the struct:

* Any integer type (usage not yet implemented)
* Any float type (usage not yet implemented)
* `String`
* `bool`, for optional flags with no argument
* `Option<T>`, for optional flags with an argument
