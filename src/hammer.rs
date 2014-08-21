/*!
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
*/

#![crate_name = "hammer"]
#![crate_type = "rlib"]
#![feature(macro_rules)]

extern crate serialize;
use serialize::{Decoder, Decodable};
use std::collections::hashmap::HashMap;

pub use usage::usage;
use usage::UsageDecoder;
use util::{canonical_field_name};

pub trait FlagConfig {
    fn config(_: Option<Self>, c: FlagConfiguration) -> FlagConfiguration {
        c
    }
}

trait FlagParse : FlagConfig {
    fn decode_flags(d: &mut FlagDecoder) -> Result<Self, HammerError>;
}

impl<T: FlagConfig + Decodable<FlagDecoder, HammerError>> FlagParse for T {
    fn decode_flags(d: &mut FlagDecoder) -> Result<T, HammerError> {
        Decodable::decode(d)
    }
}

pub trait Flags : FlagParse + UsageParse {}
impl<T: FlagParse + UsageParse> Flags for T {}


trait UsageParse : FlagConfig {
    fn decode_usage(d: &mut UsageDecoder) -> Result<Self, HammerError>;
}

impl<T: FlagConfig + Decodable<UsageDecoder, HammerError>> UsageParse for T {
    fn decode_usage(d: &mut UsageDecoder) -> Result<T, HammerError> {
        Decodable::decode(d)
    }
}

mod hammer {
    pub use super::{FlagConfiguration, FlagConfig};
}

/**
Make a struct usable by hammer by implementing `FlagConfig` for it.

Usage: `hammer_config!(TYPE ["DESCRIPTION"] [, |c| {EXPRESSION} ])`

The optional DESCRIPTION will be returned by `usage`, and the optional
EXPRESSION is useful for adding short versions of flags, etc.; see
`FlagConfiguration`.
*/
#[macro_export]
macro_rules! hammer_config(
    ($ty:ty $desc:expr , | $id:ident | { $expr:expr }) => (
        impl ::hammer::FlagConfig for $ty {
            fn config(_: Option<$ty>, $id: ::hammer::FlagConfiguration) -> ::hammer::FlagConfiguration {
                $expr.desc($desc)
            }
        }
    );

    ($ty:ty | $id:ident | { $expr:expr }) => (
        impl ::hammer::FlagConfig for $ty {
            fn config(_: Option<$ty>, $id: ::hammer::FlagConfiguration) -> ::hammer::FlagConfiguration {
                $expr
            }
        }
    );

    ($ty:ty $desc:expr) => (
        impl ::hammer::FlagConfig for $ty {
            fn config(_: Option<$ty>, c: ::hammer::FlagConfiguration) -> ::hammer::FlagConfiguration {
                c.desc($desc)
            }
        }
    );

    ($ty:ty) => (
        impl ::hammer::FlagConfig for $ty {
            fn config(_: Option<$ty>, c: ::hammer::FlagConfiguration) -> ::hammer::FlagConfiguration {
                c
            }
        }
    )
)

mod util;
mod usage;

/** Contains the configuration associated with a FlagConfig,
such as the short versions of flags and description of the program.
*/
#[deriving(Show, PartialEq)]
pub struct FlagConfiguration {
    short_aliases: HashMap<String, char>,
    description: Option<String>,
    rest_field: String
}

impl FlagConfiguration {
    pub fn new() -> FlagConfiguration {
        FlagConfiguration {
            short_aliases: HashMap::new(),
            description: None,
            rest_field: "rest".to_string()
        }
    }

    /// Add new "short" version of a flag
    ///
    /// ```flag_config.short("verbose", 'v')```
    pub fn short(mut self, string: &str, char: char) -> FlagConfiguration {
        self.short_aliases.insert(string.to_string(), char);
        self
    }

    /// Add a description
    ///
    /// ```flag_config.descr("Foo is a program to do bar")```
    pub fn desc(mut self, string: &str) -> FlagConfiguration {
        self.description = Some(string.to_string());
        self
    }

    /// Change the name of the "extra arguments" field
    ///
    /// The associated field must be of `type Vec<String>`
    ///
    /// ```flag_config.rest_field("remaining")```
    pub fn rest_field(mut self, string: &str) -> FlagConfiguration {
        self.rest_field = string.to_string();
        self
    }

    pub fn short_for(&self, field: &str) -> Option<char> {
        self.short_aliases.find_equiv(&field).map(|c| *c)
    }

    pub fn description(&self) -> Option<String> {
        self.description.as_ref().map(|d| d.clone())
    }
}

#[deriving(Show, PartialEq)]
enum DecoderState {
    Processing,
    ProcessingRest(int)
}

#[deriving(Show, PartialEq)]
pub struct FlagDecoder {
    source: Vec<String>,
    current_field: Option<String>,
    error: Option<String>,
    config: FlagConfiguration,
    state: DecoderState,
    done: bool
}

impl FlagDecoder {
    pub fn new<T: FlagConfig>(args: &[String]) -> FlagDecoder {
        let flag_config = FlagConfiguration::new();
        FlagDecoder{
            source: Vec::from_slice(args),
            current_field: None,
            error: None,
            config: FlagConfig::config(None::<T>, flag_config),
            state: Processing,
            done: false
        }
    }

    pub fn remaining(&self) -> Vec<String> {
        self.source.clone()
    }

    /*
        These helper functions encapsulate the different ways of using a field name.
        For now, this is limited to the field name prefixed by `--`, but I plan to
        add short-name configuration and `--foo=bar` support soon. These methods should
        be the only place that needs to be updated to support new forms.
    */

    fn canonical_field_name(&self) -> String {
        canonical_field_name(self.current_field.get_ref().as_slice())
    }

    fn field_pos(&self) -> Option<uint> {
        let source = &self.source;
        let aliases = &self.config.short_aliases;

        source.as_slice().position_elem(&self.canonical_field_name()).or_else(|| {
            aliases.find(self.current_field.get_ref()).and_then(|&c| {
                source.iter().position(|s| s.as_bytes()[0] == '-' as u8 && s.as_bytes()[1] == c as u8)
            })
        })
    }

    fn remove_bool_field(&mut self) {
        let pos = self.field_pos();
        self.source.remove(pos.unwrap());
    }

    fn remove_val_field(&mut self) {
        let pos = self.field_pos();

        // removes the flag and the value it's set to
        self.source.remove(pos.unwrap());
        self.source.remove(pos.unwrap());
    }
}

pub type HammerResult<T> = Result<T, HammerError>;

#[deriving(Clone, PartialEq, PartialOrd, Hash, Show)]
pub struct HammerError {
    pub message: String
}

impl HammerError {
    fn new<T>(message: String) -> HammerResult<T> {
        Err(HammerError{ message: message })
    }
}

impl Decoder<HammerError> for FlagDecoder {
    fn read_nil(&mut self) -> HammerResult<()> { unimplemented!() }

    fn read_uint(&mut self) -> HammerResult<uint> {
        match self.read_str() {
            Ok(s) => {
                match from_str(s.as_slice()) {
                    Some(i) => Ok(i),
                    None => Err(HammerError { message: format!("could not convert {} to an integer", s) })
                }
            },
            Err(e) => Err(e)
        }
    }

    // doesn't handle "too large to represent" problems. will just truncate.
    fn read_u64(&mut self) -> HammerResult<u64> { self.read_uint().map(|v| v as u64) }
    fn read_u32(&mut self) -> HammerResult<u32> { self.read_uint().map(|v| v as u32) }
    fn read_u16(&mut self) -> HammerResult<u16> { self.read_uint().map(|v| v as u16) }
    fn read_u8(&mut self) -> HammerResult<u8>   { self.read_uint().map(|v| v as u8)  }
    fn read_int(&mut self) -> HammerResult<int> { self.read_uint().map(|v| v as int) }
    fn read_i64(&mut self) -> HammerResult<i64> { self.read_uint().map(|v| v as i64) }
    fn read_i32(&mut self) -> HammerResult<i32> { self.read_uint().map(|v| v as i32) }
    fn read_i16(&mut self) -> HammerResult<i16> { self.read_uint().map(|v| v as i16) }
    fn read_i8(&mut self) -> HammerResult<i8>   { self.read_uint().map(|v| v as i8)  }

    fn read_bool(&mut self) -> HammerResult<bool> {
        match self.field_pos() {
            None => Ok(false),
            Some(_) => {
                self.remove_bool_field();
                Ok(true)
            }
        }
    }

    fn read_f64(&mut self) -> HammerResult<f64> {
        match self.read_str() {
            Ok(s) => {
                match from_str(s.as_slice()) {
                    Some(f) => Ok(f),
                    None => Err(HammerError { message: format!("could not convert {} to a float", s) })
                }
            },
            Err(e) => Err(e)
        }
    }
    fn read_f32(&mut self) -> HammerResult<f32> { self.read_f64().map(|v| v as f32) }
    fn read_char(&mut self) -> HammerResult<char> {
        match self.read_str() {
            Ok(s) => {
                if s.as_slice().char_len() == 1 {
                    Ok(s.as_slice().char_at(0))
                } else {
                    Err(HammerError { message: format!("{} is not a single character", s) })
                }
            },
            Err(e) => Err(e)
        }
    }

    fn read_str(&mut self) -> HammerResult<String> {
        match self.state {
            ProcessingRest(i) => return Ok(self.remaining()[i as uint].to_string()),
            _ => ()
        }

        let position = self.field_pos();

        if position.is_none() {
            return HammerError::new(format!("{} is required", self.canonical_field_name()));
        }

        let pos = position.unwrap();
        let val = self.source[pos + 1].clone();

        self.remove_val_field();

        Ok(val)
        /* NOTE: when Vec has an indexing method that returns an Option, do
         * this.
        match val {
            None => HammerError::new(format!("{} is missing a following string", self.canonical_field_name())),
            Some(val) => Ok(val)
        }
        */
    }

    #[allow(unused_variable)]
    fn read_struct<T>(&mut self, s_name: &str, len: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> {
        f(self)
    }

    #[allow(unused_variable)]
    fn read_struct_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> {
        assert!(!self.done, "Flag struct must not contain any fields after {}", self.config.rest_field);

        self.current_field = Some(f_name.to_string());
        f(self)
    }

    fn read_option<T>(&mut self, f: |&mut FlagDecoder, bool| -> HammerResult<T>) -> HammerResult<T> {
        match self.field_pos() {
            None => f(self, false),
            Some(_) => f(self, true)
        }
    }

    // the rest of these are pretty weird or hard to implement.

    #[allow(unused_variable)]
    fn read_enum<T>(&mut self, name: &str, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_variant<T>(&mut self, names: &[&str], f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_variant_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_struct_variant<T>(&mut self, names: &[&str], f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_struct_variant_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }

    #[allow(unused_variable)]
    fn read_tuple<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct<T>(&mut self, s_name: &str, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }

    #[allow(unused_variable)]
    fn read_seq<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> {
        let len = self.remaining().len();
        let current_field = self.current_field.as_ref().unwrap().to_string();

        if current_field.as_slice() != self.config.rest_field.as_slice() { unimplemented!() }
        self.state = ProcessingRest(-1);
        let ret = f(self, len);
        self.done = true;
        ret
    }

    #[allow(unused_variable)]
    fn read_seq_elt<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> {
        self.state = match self.state {
            ProcessingRest(i) => ProcessingRest(i + 1),
            _ => unimplemented!()
        };

        f(self)
    }

    #[allow(unused_variable)]
    fn read_map<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    fn error(&mut self, err: &str) -> HammerError { HammerError { message: err.to_string() } }
}

/**
Convert arguments into struct T

hammer_config! must be called on T beforehand.
*/
pub fn decode_args<T: FlagParse>(args: &[String]) -> HammerResult<T> {
    let mut decoder = FlagDecoder::new::<T>(args);
    FlagParse::decode_flags(&mut decoder)
}

#[cfg(test)]
mod tests {
    use super::{FlagDecoder, HammerResult, HammerError};
    use serialize::{Decoder,Decodable};

    #[deriving(Decodable, Show, PartialEq)]
    struct CompileFlags {
        color: bool,
        count: uint,
        maybe: Option<uint>,
        some_some: bool
    }

    hammer_config!(CompileFlags |c| {
        c.short("color", 'c')
    })

    #[deriving(Decodable, Show, PartialEq)]
    struct GlobalFlags {
        color: bool,
        verbose: bool,
        rest: Vec<String>
    }

    hammer_config!(GlobalFlags)

    #[deriving(Decodable, Show, PartialEq)]
    struct AliasedRest {
        color: bool,
        verbose: bool,
        remaining: Vec<String>
    }

    hammer_config!(AliasedRest |c| {
        c.short("verbose", 'v').rest_field("remaining")
    })

    #[test]
    fn test_example() {
        let args = vec!("--count".to_string(), "1".to_string(), "foo".to_string(), "-c".to_string());
        let mut decoder = FlagDecoder::new::<CompileFlags>(args.as_slice());
        let flags: CompileFlags = Decodable::decode(&mut decoder).unwrap();

        assert_eq!(decoder.remaining(), vec!("foo".to_string()));
        assert_eq!(flags, CompileFlags{ color: true, count: 1u, maybe: None, some_some: false });
    }

    #[test]
    fn test_err() {
        let mut decoder = FlagDecoder::new::<CompileFlags>(vec!().as_slice());
        let flags: HammerResult<CompileFlags> = Decodable::decode(&mut decoder);

        assert_eq!(flags, Err(HammerError { message: "--count is required".to_string() }));

        assert!(decoder.error == None, "The decoder doesn't have an error");
    }

    #[test]
    fn test_rest() {
        let args = vec!("--verbose".to_string(), "hello".to_string(), "goodbye".to_string());

        let mut decoder = FlagDecoder::new::<GlobalFlags>(args.as_slice());
        let flags: GlobalFlags = Decodable::decode(&mut decoder).unwrap();

        assert_eq!(flags, GlobalFlags { color: false, verbose: true, rest: vec!("hello".to_string(), "goodbye".to_string()) });
    }

    #[test]
    fn test_aliased_rest() {
        let args = vec!("-v".to_string(), "hello".to_string(), "goodbye".to_string());

        let mut decoder = FlagDecoder::new::<AliasedRest>(args.as_slice());
        let flags: AliasedRest = Decodable::decode(&mut decoder).unwrap();

        assert_eq!(flags, AliasedRest { color: false, verbose: true, remaining: vec!("hello".to_string(), "goodbye".to_string()) });
    }

}
