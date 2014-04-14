#![crate_id="hammer"]
#![crate_type = "rlib"]

extern crate serialize;
extern crate collections;
use serialize::Decoder;
use collections::hashmap::HashMap;

pub trait FlagConfig {
    fn config(_: Option<Self>, c: FlagConfiguration) -> FlagConfiguration {
        c
    }
}

#[deriving(Show, Eq)]
pub struct FlagConfiguration {
    short_aliases: HashMap<~str, char>
}

impl FlagConfiguration {
    pub fn new() -> FlagConfiguration {
        FlagConfiguration{ short_aliases: HashMap::new() }
    }

    pub fn short(mut self, string: ~str, char: char) -> FlagConfiguration {
        self.short_aliases.insert(string, char);
        self
    }
}

#[deriving(Show, Eq)]
pub struct FlagDecoder {
    source: Vec<~str>,
    current_field: Option<~str>,
    error: Option<~str>,
    config: FlagConfiguration
}

impl FlagDecoder {
    pub fn new<T: FlagConfig>(args: Vec<~str>) -> FlagDecoder {
        let flag_config = FlagConfiguration::new();
        FlagDecoder{ source: args, current_field: None, error: None, config: FlagConfig::config(None::<T>, flag_config) }
    }

    pub fn remaining(&self) -> Vec<~str> {
        self.source.clone()
    }

    /*
        These helper functions encapsulate the different ways of using a field name.
        For now, this is limited to the field name prefixed by `--`, but I plan to
        add short-name configuration and `--foo=bar` support soon. These methods should
        be the only place that needs to be updated to support new forms.
    */

    fn canonical_field_name(&self) -> ~str {
        let field = &self.current_field;
        format!("--{}", field.get_ref().chars().map(|c| if c == '_' {'-'} else {c}).collect::<~str>())
    }

    fn field_pos(&self) -> Option<uint> {
        let source = &self.source;
        let aliases = &self.config.short_aliases;

        source.as_slice().position_elem(&self.canonical_field_name()).or_else(|| {
            aliases.find(self.current_field.get_ref()).and_then(|&c| {
                source.iter().position(|s| s[0] == '-' as u8 && s[1] == c as u8)
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

#[deriving(Clone, Eq, Ord, Hash, Show)]
pub struct HammerError {
    pub message: ~str
}

impl HammerError {
    fn new<T>(message: ~str) -> HammerResult<T> {
        Err(HammerError{ message: message })
    }
}

impl Decoder<HammerError> for FlagDecoder {
    fn read_nil(&mut self) -> HammerResult<()> { unimplemented!() }

    fn read_uint(&mut self) -> HammerResult<uint> {
        let position = self.field_pos();

        if position.is_none() {
            return HammerError::new(format!("{} is required", self.canonical_field_name()));
        }

        let pos = position.unwrap();
        let val = from_str(self.source.get(pos + 1).as_slice());

        self.remove_val_field();

        match val {
            None => HammerError::new(format!("{} is missing a following integer", self.canonical_field_name())),
            Some(val) => Ok(val)
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
                match from_str(s) {
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
                if s.char_len() == 1 {
                    Ok(s.char_at(0))
                } else {
                    Err(HammerError { message: format!("{} is not a single character", s) })
                }
            },
            Err(e) => Err(e)
        }
    }

    fn read_str(&mut self) -> HammerResult<~str> {
        let position = self.field_pos();

        if position.is_none() {
            return HammerError::new(format!("{} is required", self.canonical_field_name()));
        }

        let pos = position.unwrap();
        let val = from_str(self.source.get(pos + 1).as_slice());

        self.remove_val_field();

        match val {
            None => HammerError::new(format!("{} is missing a following string", self.canonical_field_name())),
            Some(val) => Ok(val)
        }
    }

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
    fn read_struct<T>(&mut self, s_name: &str, len: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> {
        f(self)
    }

    #[allow(unused_variable)]
    fn read_struct_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> {
        self.current_field = Some(f_name.to_owned());
        f(self)
    }

    #[allow(unused_variable)]
    fn read_tuple<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct<T>(&mut self, s_name: &str, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }

    fn read_option<T>(&mut self, f: |&mut FlagDecoder, bool| -> HammerResult<T>) -> HammerResult<T> {
        match self.field_pos() {
            None => f(self, false),
            Some(_) => f(self, true)
        }
    }

    #[allow(unused_variable)]
    fn read_seq<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_seq_elt<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map<T>(&mut self, f: |&mut FlagDecoder, uint| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> HammerResult<T>) -> HammerResult<T> { unimplemented!() }
}

#[cfg(test)]
mod tests {
    use super::{FlagConfig, FlagConfiguration, FlagDecoder};
    use serialize::{Decoder,Decodable};

    #[deriving(Decodable, Show, Eq)]
    struct CompileFlags {
        color: bool,
        count: uint,
        maybe: Option<uint>,
        some_some: bool
    }

    impl FlagConfig for CompileFlags {
        fn config(_dummy_self: Option<CompileFlags>, c: FlagConfiguration) -> FlagConfiguration {
            c.short("color", 'c')
        }
    }

    #[test]
    fn test_example() {
        let args = vec!("--count".to_owned(), "1".to_owned(), "foo".to_owned(), "-c".to_owned());
        let mut decoder = FlagDecoder::new::<CompileFlags>(args);
        let flags: CompileFlags = Decodable::decode(&mut decoder);

        assert_eq!(decoder.remaining(), vec!("foo".to_owned()));
        assert_eq!(flags, CompileFlags{ color: true, count: 1u, maybe: None, some_some: false });
    }

    #[test]
    fn test_err() {
        let mut decoder = FlagDecoder::new::<CompileFlags>(~[]);
        let flags: CompileFlags = Decodable::decode(&mut decoder);

        assert!(decoder.error != None, "The decoder has an error");
    }

    // TODO: value flags (like --count) should produce an error, not fail! if they are used
    // without a following value
}

