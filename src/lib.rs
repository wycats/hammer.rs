#[feature(macro_rules)];
#[crate_id="hammer"];

extern crate serialize;
extern crate collections;
use serialize::{Decoder,Decodable};
use collections::hashmap::HashMap;

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

trait FlagConfig {
  fn config(_dummy_self:Option<Self>, c: FlagConfiguration) -> FlagConfiguration;
}

#[deriving(Show, Eq)]
struct FlagConfiguration {
  short_aliases: HashMap<~str, char>
}

impl FlagConfiguration {
  fn new() -> FlagConfiguration {
    FlagConfiguration{ short_aliases: HashMap::new() }
  }

  fn short(mut self, string: &str, char: char) -> FlagConfiguration {
    self.short_aliases.insert(string.to_owned(), char);
    self
  }
}

#[deriving(Show, Eq)]
struct FlagDecoder {
  source: ~[~str],
  current_field: Option<~str>,
  error: Option<~str>,
  config: FlagConfiguration
}

impl FlagDecoder {
  pub fn new<T: FlagConfig>(args: &[~str]) -> FlagDecoder {
    let flag_config = FlagConfiguration::new();
    FlagDecoder{ source: args.to_owned(), current_field: None, error: None, config: FlagConfig::config(None::<T>, flag_config) }
  }

  pub fn remaining(&self) -> ~[~str] {
    self.source.clone()
  }

  /**
    These helper functions encapsulate the different ways of using a field name.
    For now, this is limited to the field name prefixed by `--`, but I plan to
    add short-name configuration and `--foo=bar` support soon. These methods should
    be the only place that needs to be updated to support new forms.
  */

  fn canonical_field_name(&self) -> ~str {
    let field = &self.current_field;
    let canonical: ~str = field.get_ref().chars().map(|c| if c == '_' {'-'} else {c}).collect();
    (format!("--{}", canonical)).to_owned()
  }

  fn field_pos(&self) -> Option<uint> {
    let source = &self.source;
    let aliases = &self.config.short_aliases;

    source.position_elem(&self.canonical_field_name()).or_else(|| {
      aliases.find(self.current_field.get_ref()).and_then(|char| {
        source.position_elem(&format!("-{}", char))
      })
    })
  }

  fn remove_bool_field(&mut self) {
    let pos = self.field_pos();
    let source = &mut self.source;

    source.remove(pos.unwrap());
  }

  fn remove_val_field(&mut self) {
    let pos = self.field_pos();
    let source = &mut self.source;

    source.remove(pos.unwrap());
    source.remove(pos.unwrap());
  }
}

impl Decoder for FlagDecoder {
  fn read_nil(&mut self) { unimplemented!() }

  fn read_uint(&mut self) -> uint {
    let position = self.field_pos();

    if position.is_none() {
      self.error = Some(format!("{} is required", self.canonical_field_name()));
      return 0;
    }

    let pos = position.unwrap();
    let val = from_str(self.source[pos + 1]);

    self.remove_val_field();

    match val {
      None => {
        self.error = Some(format!("{} is missing a following integer", self.canonical_field_name()));
        0
      },
      Some(val) => val
    }

  }

  fn read_u64(&mut self) -> u64 { unimplemented!() }
  fn read_u32(&mut self) -> u32 { unimplemented!() }
  fn read_u16(&mut self) -> u16 { unimplemented!() }
  fn read_u8(&mut self) -> u8 { unimplemented!() }
  fn read_int(&mut self) -> int { unimplemented!() }
  fn read_i64(&mut self) -> i64 { unimplemented!() }
  fn read_i32(&mut self) -> i32 { unimplemented!() }
  fn read_i16(&mut self) -> i16 { unimplemented!() }
  fn read_i8(&mut self) -> i8 { unimplemented!() }

  fn read_bool(&mut self) -> bool {
    match self.field_pos() {
      None => false,
      Some(pos) => {
        self.remove_bool_field();
        true
      }
    }
  }

  fn read_f64(&mut self) -> f64 { unimplemented!() }
  fn read_f32(&mut self) -> f32 { unimplemented!() }
  fn read_char(&mut self) -> char { unimplemented!() }
  fn read_str(&mut self) -> ~str { unimplemented!() }
  fn read_enum<T>(&mut self, name: &str, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
  fn read_enum_variant<T>(&mut self, names: &[&str], f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_enum_variant_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
  fn read_enum_struct_variant<T>(&mut self, names: &[&str], f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_enum_struct_variant_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }

  fn read_struct<T>(&mut self, s_name: &str, len: uint, f: |&mut FlagDecoder| -> T) -> T {
    f(self)
  }

  fn read_struct_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut FlagDecoder| -> T) -> T {
    self.current_field = Some(f_name.to_owned());
    f(self)
  }

  fn read_tuple<T>(&mut self, f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_tuple_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
  fn read_tuple_struct<T>(&mut self, s_name: &str, f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_tuple_struct_arg<T>(&mut self, a_idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }

  fn read_option<T>(&mut self, f: |&mut FlagDecoder, bool| -> T) -> T {
    match self.field_pos() {
      None => f(self, false),
      Some(_) => f(self, true)
    }
  }

  fn read_seq<T>(&mut self, f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_seq_elt<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
  fn read_map<T>(&mut self, f: |&mut FlagDecoder, uint| -> T) -> T { unimplemented!() }
  fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
  fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut FlagDecoder| -> T) -> T { unimplemented!() }
}

fn main() {
  let mut decoder = FlagDecoder::new::<CompileFlags>(std::os::args().tail());
  let flags: CompileFlags = Decodable::decode(&mut decoder);

  let remaining = decoder.remaining();

  match decoder.error {
    None => {
      println!("{:?}", flags);
      println!("remaining: {:?}", remaining);
    }
    Some(err) => println!("{}", err)
  }

}
