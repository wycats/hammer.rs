use std::default::Default;
use serialize::{Decoder, Decodable};

use util::canonical_field_name;
use {FlagConfig, FlagConfiguration, HammerError};

#[deriving(PartialEq, Clone, Show)]
struct FieldUsage {
    canonical: String,
    alias: Option<char>,
    optional: bool
}

impl FieldUsage {
    fn new(canonical: &str) -> FieldUsage {
        FieldUsage { canonical: canonical.to_str(), alias: None, optional: false }
    }

    fn alias(&mut self, alias: char) {
        self.alias = Some(alias);
    }

    fn optional(&mut self) {
        self.optional = true;
    }
}

pub struct UsageDecoder {
    config: FlagConfiguration,
    current_field: Option<FieldUsage>,
    fields: Vec<FieldUsage>,
}

struct SwallowUsage;

hammer_config!(SwallowUsage)

impl UsageDecoder {
    pub fn new<T: FlagConfig>(dummy: Option<T>) -> UsageDecoder {
        let flag_config = FlagConfiguration::new();

        UsageDecoder {
            config: FlagConfig::config(dummy, flag_config),
            current_field: None,
            fields: vec!()
        }
    }

    fn optional(&mut self) {
        match self.current_field {
            Some(ref mut f) => f.optional(),
            None => fail!("No current field")
        }
    }

    fn field(&mut self) {
        self.fields.push(self.current_field.take_unwrap())
    }
}

type UsageResult<T> = Result<T, HammerError>;

fn default<T: Default>() -> UsageResult<T> {
    Ok(Default::default())
}

impl Decoder<HammerError> for UsageDecoder {
    fn read_nil(&mut self) -> UsageResult<()> { unimplemented!() }

    fn read_uint(&mut self) -> UsageResult<uint> {
        unimplemented!()
    }

    // doesn't handle "too large to represent" problems. will just truncate.
    fn read_u64(&mut self) -> UsageResult<u64> { self.read_uint().map(|v| v as u64) }
    fn read_u32(&mut self) -> UsageResult<u32> { self.read_uint().map(|v| v as u32) }
    fn read_u16(&mut self) -> UsageResult<u16> { self.read_uint().map(|v| v as u16) }
    fn read_u8(&mut self) -> UsageResult<u8>   { self.read_uint().map(|v| v as u8)  }
    fn read_int(&mut self) -> UsageResult<int> { self.read_uint().map(|v| v as int) }
    fn read_i64(&mut self) -> UsageResult<i64> { self.read_uint().map(|v| v as i64) }
    fn read_i32(&mut self) -> UsageResult<i32> { self.read_uint().map(|v| v as i32) }
    fn read_i16(&mut self) -> UsageResult<i16> { self.read_uint().map(|v| v as i16) }
    fn read_i8(&mut self) -> UsageResult<i8>   { self.read_uint().map(|v| v as i8)  }

    fn read_bool(&mut self) -> UsageResult<bool> {
        self.optional();
        self.field();
        default()
    }

    fn read_f64(&mut self) -> UsageResult<f64> {
        default()
    }

    fn read_f32(&mut self) -> UsageResult<f32> { self.read_f64().map(|v| v as f32) }

    fn read_char(&mut self) -> UsageResult<char> {
        self.field();
        default()
    }

    fn read_str(&mut self) -> UsageResult<String> {
        self.field();
        default()
    }

    #[allow(unused_variable)]
    fn read_struct<T>(&mut self, s_name: &str, len: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> {
        f(self)
    }

    #[allow(unused_variable)]
    fn read_struct_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> {
        let name = canonical_field_name(f_name);
        let mut field = FieldUsage::new(name.as_slice());

        self.config.short_for(f_name).map(|short| {
            field.alias(short);
        });

        self.current_field = Some(field);

        if f_name == "rest" {
            f(&mut UsageDecoder::new(None::<SwallowUsage>))
        } else {
            f(self)
        }
    }

    fn read_option<T>(&mut self, f: |&mut UsageDecoder, bool| -> UsageResult<T>) -> UsageResult<T> {
        self.optional();
        f(self, true)
    }

    // the rest of these are pretty weird or hard to implement.

    #[allow(unused_variable)]
    fn read_enum<T>(&mut self, name: &str, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_variant<T>(&mut self, names: &[&str], f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_variant_arg<T>(&mut self, a_idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_struct_variant<T>(&mut self, names: &[&str], f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_enum_struct_variant_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }

    #[allow(unused_variable)]
    fn read_tuple<T>(&mut self, f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_arg<T>(&mut self, a_idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct<T>(&mut self, s_name: &str, f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_tuple_struct_arg<T>(&mut self, a_idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }

    #[allow(unused_variable)]
    fn read_seq<T>(&mut self, f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> {
        f(self, 0)
    }

    #[allow(unused_variable)]
    fn read_seq_elt<T>(&mut self, idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> {
        unimplemented!()
    }

    #[allow(unused_variable)]
    fn read_map<T>(&mut self, f: |&mut UsageDecoder, uint| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
    #[allow(unused_variable)]
    fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut UsageDecoder| -> UsageResult<T>) -> UsageResult<T> { unimplemented!() }
}

pub fn usage<T: Decodable<UsageDecoder, HammerError> + FlagConfig>(force_indent: bool) -> (Option<String>, String) {
    let mut decoder: UsageDecoder = UsageDecoder::new(None::<T>);
    let _: Result<T, HammerError> = Decodable::decode(&mut decoder);

    let fields = decoder.fields;
    let desc = decoder.config.description();
    let options = print_usage(fields.as_slice(), force_indent);

    (desc, options)
}

fn print_usage(fields: &[FieldUsage], force_indent: bool) -> String {
    let mut out = String::new();
    let shorthands = fields.iter().any(|f| f.alias.is_some());

    let indent = if force_indent || shorthands {
        "    "
    } else {
        ""
    };

    let (optional, mandatory) = Vec::from_slice(fields).partition(|f| f.optional);

    out.push_str(print_fields(mandatory.as_slice(), indent, |f| f.to_str()).as_slice());
    out.push_str(print_fields(optional.as_slice(), indent, |f| format!("[{}]", f)).as_slice());

    out
}

fn print_fields(fields: &[FieldUsage], indent: &str, format: |&str| -> String) -> String {
    let mut out = String::new();

    for field in fields.iter() {
        let shorthand = field.alias
            .map(|a| format!("-{}, ", a))
            .unwrap_or(indent.to_str());

        let longhand = format(field.canonical.as_slice());

        out.push_str(format!("{}{}\n", shorthand, longhand).as_slice());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::usage;

    #[allow(dead_code)]
    #[deriving(Decodable)]
    struct MixedOptions {
        color: Option<String>,
        line_count: String,
        verbose: bool,
        rest: Vec<String>
    }

    hammer_config!(MixedOptions |c| {
        c.short("verbose", 'v')
    })

    #[allow(dead_code)]
    #[deriving(Decodable)]
    struct NoShorthandOptions {
        color: Option<String>,
        line_count: String,
        verbose: bool,
        rest: Vec<String>
    }

    hammer_config!(NoShorthandOptions)

    #[test]
    fn test_mixed_usage() {
        assert_eq!(usage::<MixedOptions>(false), "    --line-count\n    [--color]\n-v, [--verbose]\n".to_str())
    }

    #[test]
    fn test_no_shorthand_usage() {
        assert_eq!(usage::<NoShorthandOptions>(false), "--line-count\n[--color]\n[--verbose]\n".to_str())
    }
}
