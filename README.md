# xml_to_json_rs - XML to JSON rust library
Rust library to convert an XML string to serde_json::Value

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
