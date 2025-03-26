# XML to JSON rust library - xml_to_json_rs
Rust library to convert an XML string to serde_json::Value

This library is designed to be simple and create serde_json::Value.

It supports
- Strings
- Arrays
- Objects

Numbers and Booleans will just be parsed as String, as they also look the same in XML.
The inner text attributes will be wrapped by default into `#text` and attributes will 
get an `@`, for example `@href`. By this, the result will be consistent and can 
be transformed back to XML. This library is not using `$text` as this creates 
issues with MongoDB. 

To transform the JSON back to XML, you can use `prepare_for_quick_xml` and `quick-xml`:
```rust

let simple_xml = "<a><b>simple</b></a>";
let parser = XmlToJson::default();
let json_value = parser.xml_to_json(xml).unwrap();
let comp_value = parser.prepare_for_quick_xml(json_value);
assert_eq!(xml, quick_xml::se::to_string_with_root("a", &comp_value).unwrap());


```

Check the tests in `src/libs.rs` to see further usage examples.

Usage:
```rust
    let simple_xml = "<a><b>simple</b></a>";
    assert_eq!(
        XmlToJson::default()
            .with_root()
            .xml_to_json(simple_xml)
            .unwrap(),
        json!({ "a": {"b": { "#text": "simple" } } })
    );

    let nested_with_xml_with_attributes =
        "<a><b href=\"#self\">simple</b><b><c class=\"my_class\"><d>D</d><d>1</d></c></b></a>";
    assert_eq!(
        XmlToJson::default()
            .xml_to_json(nested_with_xml_with_attributes)
            .unwrap(),
        json!({ "b": [{ "@href": "#self", "#text": "simple" }  , {"c": { "@class": "my_class", "d": [{ "#text": "D" }, { "#text": "1" }] } }   ] })
    );
```
