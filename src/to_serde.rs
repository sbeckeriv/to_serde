pub mod serde_xml {
    use inflector::Inflector;
    use itertools::Itertools;
    use serde_xml_rs::EventReader;
    use std::{
        collections::{HashMap},
        fmt::Display,
        process::exit,
    };
    use xml::reader::XmlEvent::EndDocument;

    fn guess_type(input: &str) -> ValueTypeOption {
        let val = if let Ok(_) = input.parse::<i64>() {
            ValueTypes::Int
        } else if let Ok(_) = input.parse::<f64>() {
            ValueTypes::Float
        } else if let Ok(_) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
            ValueTypes::Timestamp("DateTime".into())
        } else if let Ok(_) = chrono::DateTime::parse_from_rfc3339(input) {
            ValueTypes::Timestamp("rfc3339".into())
        } else if let Ok(_) = chrono::DateTime::parse_from_rfc2822(input) {
            ValueTypes::Timestamp("rfc282".into())
        } else if let Ok(_) = url::Url::parse(input) {
            ValueTypes::Url
        } else {
            ValueTypes::String
        };
        ValueTypeOption::Required(val)
    }
    #[derive(Debug, Clone)]
    enum ValueTypeOption {
        Optional(ValueTypes),
        Required(ValueTypes),
    }
    impl ValueTypeOption {
        pub fn optional(&mut self) {
            *self = match self {
                ValueTypeOption::Optional(value) => Self::Optional(value.clone()),
                ValueTypeOption::Required(value) => Self::Optional(value.clone()),
            };
        }
    }

    impl Display for ValueTypeOption {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let val = match self {
                ValueTypeOption::Optional(value) if value == &ValueTypes::None => value.to_string(),
                ValueTypeOption::Optional(value) => format!("Option<{value}>"),
                ValueTypeOption::Required(value) => value.to_string(),
            };
            write!(f, "{val}")
        }
    }
    impl Default for ValueTypeOption {
        fn default() -> Self {
            ValueTypeOption::Required(ValueTypes::default())
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    enum ValueTypes {
        None,
        String,
        Int,
        Float,
        Url,
        Timestamp(String),
    }
    impl Display for ValueTypes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let val = match self {
                ValueTypes::String => "String",
                ValueTypes::None => "Option<String>",
                ValueTypes::Int => "u64",
                ValueTypes::Float => "f64",
                ValueTypes::Url => "url::Url",
                ValueTypes::Timestamp(str) if *str == "DateTime".to_string() => "NaiveDateTime",
                ValueTypes::Timestamp(_) => "DateTime<FixedOffset>",
            };
            write!(f, "{val}")
        }
    }
    impl Default for ValueTypes {
        fn default() -> Self {
            ValueTypes::None
        }
    }
    #[derive(Debug, Default, Clone)]
    pub struct Item {
        name: String,
        value_type: ValueTypeOption,
        attributes: HashMap<String, ValueTypeOption>,
    }

    impl Eq for Item {}

    impl Ord for Item {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.name.cmp(&other.name)
        }
    }
    impl PartialEq for Item {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    impl PartialOrd for Item {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.name.partial_cmp(&other.name)
        }
    }
    fn merge_attributes(
        original: &HashMap<String, ValueTypeOption>,
        other: &HashMap<String, ValueTypeOption>,
    ) -> HashMap<String, ValueTypeOption> {
        let mut new_self = original.clone();
        new_self.extend(other.clone());
        // if the key is not in the other hash map make it optional. this was the best i could figure out.
        // without ownership problems around the use of keys()
        for k in original.keys() {
            if !other.contains_key(k) {
                if let Some(mut v) = new_self.remove(k) {
                    v.optional();
                    new_self.insert(k.to_string(), v);
                }
            }
        }

        for k in other.keys() {
            if !original.contains_key(k) {
                if let Some(mut v) = new_self.remove(k) {
                    v.optional();
                    new_self.insert(k.to_string(), v);
                }
            }
        }
        new_self
    }
    impl Item {
        pub fn merge(&self, other: &Item) -> Item {
            let mut new_self = self.clone();
            new_self.attributes = merge_attributes(&self.attributes, &other.attributes);
            new_self
        }

        // (struct line, new struct)
        // rename forbidden names type,struct
        pub fn as_serde(&self, mod_name: &Option<String>) -> (String, Option<String>) {
            let val_name = if self.attributes.is_empty() {
                format!("{}", self.value_type)
            } else {
                let mod_name = mod_name
                    .as_ref()
                    .map(|f| format!("{f}::"))
                    .unwrap_or_default();
                format!("{mod_name}{}Element", self.name.to_pascal_case())
            };

            let val = val_name;
            let new_struct = if self.attributes.is_empty() {
                None
            } else {
                let struct_name = format!("{}Element", self.name.to_pascal_case());
                let mut st = format!(
                    "\n#[derive(Debug, Deserialize)]\npub struct {struct_name} {{
  #[serde(rename = \"$value\")]
  content_xml: {},\n",
                    self.value_type
                );
                for (k, v) in self.attributes.iter() {
                    st.push_str(&format!("{}: {v},\n", name_check(&k.to_snake_case())));
                }
                st.push_str("\n}\n");
                Some(st)
            };

            (format!("{}: {}", self.name, val), new_struct)
        }
    }
    #[derive(Debug, Default, Clone)]
    pub struct XmlElement {
        name: String,
        attributes: HashMap<String, ValueTypeOption>,
        items: Vec<Item>,
        value_type: ValueTypeOption,
        elements: Vec<XmlElement>,
    }
    pub fn name_check(name: &str) -> String {
        match name {
            "type" => format!("  #[serde(rename = \"{name}\")]\n  {name}_xml"),
            _ => name.to_owned(),
        }
    }
    impl XmlElement {
        pub fn merge(&self, other: &XmlElement) -> Self {
            let mut new_self = self.clone();

            new_self.attributes = merge_attributes(&self.attributes, &other.attributes);

            new_self.items.append(&mut other.items.clone());
            new_self.items.sort();
            new_self.items = new_self
                .items
                .iter()
                .group_by(|s| s.name.clone())
                .into_iter()
                .map(|(_, data)| {
                    let mut data = data.collect::<Vec<_>>();
                    let mut first = data.remove(0).clone();
                    if data.len() == 0 {
                        first
                    } else {
                        data.iter().for_each(|i| {
                            first = first.merge(&i);
                        });
                        first
                    }
                })
                .collect::<Vec<_>>();

            new_self.elements.append(&mut other.elements.clone());
            new_self
        }
        pub fn as_serde(&self) -> String {
            let mut new_structs = Vec::new();
            let mut other_structs = Vec::new();
            let mut el = format!(
                "\n#[derive(Debug, Deserialize)]\npub struct {} {{\n",
                self.name.to_pascal_case()
            );
            let mod_name = self.name.to_snake_case();
            for item in self.items.iter() {
                let (v, s) = item.as_serde(&Some(mod_name.clone()));
                new_structs.push(s);
                el.push_str(&format!("  {v},\n"));
            }
            if let Some(els) = self.elements.first() {
                let mut els = els.clone();
                for e in self.elements.iter().skip(1) {
                    els = els.merge(&e);
                }

                let name = els.name.to_pascal_case();
                let snake = &name.to_snake_case();
                el.push_str(&format!("  {}: Vec<{name}>,\n", name_check(&snake)));
                other_structs.push(Some(els.as_serde()));
            }
            el.push_str("\n}\n");
            // check for
            let new_structs = new_structs
                .into_iter()
                .filter(|s| s.is_some())
                .map(|s| s.unwrap())
                .collect::<Vec<_>>();
            if !new_structs.is_empty() {
                el.push_str(&format!(
                    "\npub mod {mod_name}{{\nuse super::*;\n{}\n}}",
                    new_structs.join("\n\n")
                ));
            }

            let other_structs = other_structs
                .into_iter()
                .filter(|s| s.is_some())
                .map(|s| s.unwrap())
                .collect::<Vec<_>>();

            if !other_structs.is_empty() {
                el.push_str(&format!("{}", other_structs.join("\n\n")));
            }
            el
        }
    }

    pub fn parse_xml_file(xml: &str) -> String {
        let mut reader = EventReader::from_str(xml);
        let root = XmlElement::default();
        let mut open = vec![root];
        while let Ok(el) = reader.next() {
            if el == EndDocument {
                break;
            }
            match el {
                xml::reader::XmlEvent::Characters(input) => {
                    let mut cur = open.pop().expect("chars called without current element");
                    cur.value_type = guess_type(&input);
                    open.push(cur)
                }
                xml::reader::XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _namespace,
                } => {
                    let mut active = XmlElement::default();
                    active.name = name.local_name;
                    let mut att_list = HashMap::new();
                    for att in attributes.iter() {
                        att_list.insert(att.name.to_string(), guess_type(&att.value));
                    }
                    active.attributes = att_list;

                    open.push(active);
                }
                xml::reader::XmlEvent::EndElement { name } => {
                    let cur = open.pop().expect("close found but nothing in the queue!");
                    if cur.name != name.local_name {
                        eprintln!("Stack error named {} did not match top of the stack {}  \nrest:{open:?}", name.to_string(), cur.name);
                        exit(2);
                    }
                    if let Some(e) = open.last_mut() {
                        if cur.elements.is_empty() && cur.items.is_empty() {
                            let mut item = Item::default();
                            item.name = cur.name;
                            item.attributes = cur.attributes;
                            item.value_type = cur.value_type;
                            e.items.push(item);
                        } else {
                            e.elements.push(cur);
                        }
                    }
                }
                _ => {}
            }
        }
        let root = open.pop().expect("should be root");
        if !open.is_empty() {
            eprintln!("The stack still has elements {open:?}");
            exit(2)
        }
        format!("{}", root.elements.first().unwrap().as_serde())
    }
}
