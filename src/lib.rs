//  XML to JSON rust library
//  © Copyright 2024, by Marco Mengelkoch
//  Licensed under MIT License, see License file for more details
//  git clone https://github.com/marcomq/xml_to_json_rs

use roxmltree;
use serde_json::{Map, Value};

pub struct XmlToJson {
    with_root: bool,
    text_name: String,
    attribute_prefix: String,
}

impl Default for XmlToJson {
    fn default() -> Self {
        XmlToJson {
            with_root: false,
            text_name: "#text".to_string(),
            attribute_prefix: "@".to_string(), // you can't serialize it again to XML when changing this
        }
    }
}

impl XmlToJson {
    /// Parse XML string and return serde_json Value
    pub fn xml_to_json(&self, xml: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(xml)?;
        let root = doc.root_element();
        Ok(self.parse_root(&root).unwrap_or(Value::Null))
    }

    /// parse with XML root, default is false as quick-xml usually doesn't parse the root
    pub fn with_root(mut self) -> Self {
        self.with_root = true;
        self
    }
    /// change name of inner XML value
    pub fn with_text_name(mut self, text_name: &str) -> Self {
        self.text_name = String::from(text_name);
        self
    }

    /// Renames #text into $text, so that the JSON can be used by quick-xml
    pub fn prepare_for_quick_xml(self, input: Value) -> Value {
        Self::rename_keys(input, &self.text_name, "$text")
    }

    // You may need to rename #text to $text to serialize it again to xml
    pub fn rename_keys(input: Value, old_key: &str, new_key: &str) -> Value {
        match input {
            Value::Object(mut map_string_val) => {
                if let Some(val) = map_string_val.remove(old_key) {
                    map_string_val.insert(new_key.into(), val);
                }
                map_string_val
                    .iter_mut()
                    .for_each(|val| *val.1 = Self::rename_keys(val.1.take(), old_key, new_key));
                map_string_val.into()
            }
            Value::Array(mut array_val) => {
                array_val
                    .iter_mut()
                    .for_each(|val| *val = Self::rename_keys(val.take(), old_key, new_key));
                array_val.into()
            }
            other => other,
        }
    }

    fn parse_root(&self, node: &roxmltree::Node) -> Option<Value> {
        if self.with_root {
            let mut root = Map::new();
            root.insert(
                node.tag_name().name().to_string(),
                self.parse_node(node).unwrap_or(Value::Null),
            );
            Some(Value::Object(root))
        } else {
            self.parse_node(node)
        }
    }

    fn parse_node(&self, node: &roxmltree::Node) -> Option<Value> {
        let mut elements = Map::new();
        if let Some(text) = node.text() {
            elements.insert(self.text_name.clone(), Value::String(text.trim().into()));
        }
        for attr in node.attributes() {
            let key = [&self.attribute_prefix, attr.name()].concat();
            let val = attr.value().trim().into();
            elements.insert(key, Value::String(val));
        }
        for child in node.children() {
            let name = child.tag_name().name().to_string();
            if !name.is_empty() {
                if let Some(child_val) = self.parse_node(&child) {
                    // check if val already exists
                    if let Some(found) = elements.get_mut(&name) {
                        if let Some(array) = found.as_array_mut() {
                            array.push(child_val);
                        } else {
                            // remove old value and convert to array
                            let new_val = match elements.remove(&name) {
                                None => vec![child_val],
                                Some(old_val) => vec![old_val, child_val],
                            };
                            elements.insert(name, Value::Array(new_val));
                        }
                    } else {
                        elements.insert(name, child_val);
                    }
                }
            }
        }
        if elements.is_empty() {
            None
        } else {
            Some(Value::Object(elements))
        }
    }
}

#[test]
fn test_basic_xml_to_json() {
    use serde_json::json;

    let simple_xml = "<a><b>simple</b></a>";
    assert_eq!(
        XmlToJson::default().xml_to_json(simple_xml).unwrap(),
        json!({ "b": { "#text": "simple" } })
    );
}

#[test]
fn test_setup_xml_to_json() {
    use serde_json::json;

    let simple_xml = "<a><b>simple</b></a>";
    assert_eq!(
        XmlToJson::default()
            .with_root()
            .xml_to_json(simple_xml)
            .unwrap(),
        json!({ "a": {"b": { "#text": "simple" } } })
    );
    assert_eq!(
        XmlToJson::default()
            .with_text_name("$text")
            .xml_to_json(simple_xml)
            .unwrap(),
        json!({"b": { "$text": "simple" } })
    );
    assert_eq!(
        XmlToJson::default()
            .with_root()
            .with_text_name("$text")
            .xml_to_json(simple_xml)
            .unwrap(),
        json!({ "a": {"b": { "$text": "simple" } } })
    );
}

#[test]
fn test_extended_xml_to_json() {
    use serde_json::json;
    let array_xml = "<a><b>simple</b><b>2</b></a>";
    assert_eq!(
        XmlToJson::default().xml_to_json(array_xml).unwrap(),
        json!({ "b": [{ "#text": "simple" }, { "#text": "2" }] })
    );

    let nested_xml = "<a><b>simple</b><b><c><d>D</d></c></b></a>";
    assert_eq!(
        XmlToJson::default().xml_to_json(nested_xml).unwrap(),
        json!({ "b": [{ "#text": "simple" }, {"c": {"d": { "#text": "D" }}}] })
    );

    let nested_with_xml_with_attributes =
        "<a><b href=\"#self\">simple</b><b><c class=\"my_class\"><d>D</d><d>1</d></c></b></a>";
    assert_eq!(
        XmlToJson::default()
            .xml_to_json(nested_with_xml_with_attributes)
            .unwrap(),
        json!({ "b": [{ "@href": "#self", "#text": "simple" }  , {"c": { "@class": "my_class", "d": [{ "#text": "D" }, { "#text": "1" }] } }   ] })
    );
}

#[test]
fn test_serde_xml_to_json_to_xml() {
    let xml =
        "<a><b href=\"#self\">simple</b><b><c class=\"my_class\"><d>D</d><d>1</d></c></b></a>";
    let parser = XmlToJson::default();
    let json_value = parser.xml_to_json(xml).unwrap();
    let comp_value = parser.prepare_for_quick_xml(json_value);
    assert_eq!(xml, quick_xml::se::to_string_with_root("a", &comp_value).unwrap());
}
