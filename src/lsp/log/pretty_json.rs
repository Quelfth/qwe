
use serde_json::{Value, Map};

use crate::grapheme::GraphemeExt as _;

const INDENT: u32 = 1;

pub fn pretty_json(value: &Value) -> String {
    let mut cx = PrettyContext::default();

    match value {
        Value::Null => return "= null".to_owned(),
        Value::Bool(value) => return format!("= {value}"),
        Value::Number(value) => return format!("= {value}"),
        Value::String(value) => pretty_string(&mut cx, 0, value),
        Value::Array(values) => pretty_array(&mut cx, values),
        Value::Object(map) => pretty_map(&mut cx, map),
    }

    cx.string
}

#[derive(Default)]
struct PrettyContext {
    string: String,
    indent: u32,
}

impl PrettyContext {
    pub fn write_indent(&mut self) {
        self.align(self.indent * INDENT)
    }

    pub fn write(&mut self, string: &str) {
        self.string += string;
    }

    pub fn align(&mut self, amount: u32) {
        for _ in 0..amount {
            self.string += " ";
        }
    }

    pub fn newline(&mut self) {
        self.string += "\n";
    }
}

fn pretty_string(cx: &mut PrettyContext, inset: u32, string: &str) {
    let mut lines = string.lines();
    let Some(first_line) = lines.next() else {
        cx.write(":");
        return
    };
    cx.write(&format!(": {first_line}\n"));
    for line in lines {
        cx.write_indent();
        cx.align(inset + 2);
        cx.write(line);
        cx.newline();
    }
}

fn pretty_array(cx: &mut PrettyContext, values: &[Value]) {
    pretty_kv_list(cx, 0, values.iter().map(|value| ("", value)))
}

fn pretty_map(cx: &mut PrettyContext, map: &Map<String, Value>) {
    let inset = map.keys().map(|k| k.len()).max().unwrap_or_default() as u32;
    pretty_kv_list(cx, inset + 1, map.into_iter().map(|(k, v)| (&**k, v)));
}

fn pretty_kv_list<'a>(cx: &mut PrettyContext, inset: u32, list: impl IntoIterator<Item = (&'a str, &'a Value)>) {
    for (k, v) in list {
        cx.write_indent();
        cx.write(k);
        let k_columns = k.graphemes().map(|g| g.columns().inner() as u32).sum::<u32>();
        cx.align(inset - k_columns);
        match v {
            Value::Null => cx.write("= null\n"),
            Value::Bool(bool) => cx.write(&format!("= {bool}\n")),
            Value::Number(number) => cx.write(&format!("= {number}\n")),
            Value::String(string) => {
                pretty_string(cx, inset, string);
            },
            Value::Array(values) => {
                cx.write("=\n");
                cx.indent += 1;
                pretty_array(cx, values);
                cx.indent -= 1;
            },
            Value::Object(map) => {
                cx.write("=\n");
                cx.indent += 1;
                pretty_map(cx, map);
                cx.indent -= 1;
            },
        }
    }
}
