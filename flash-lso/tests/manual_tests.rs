use flash_lso::types::{AMFVersion, DateValue, Element, Lso, ObjectId, Value};

#[test]
pub fn test_reference_to_nan_date() {
    let mut lso = Lso::new_empty("a", AMFVersion::AMF3);
    lso.body.push(Element::new(
        "d",
        Value::Date {
            id: ObjectId(0),
            data: DateValue {
                time: f64::NAN,
                timezone_or_utc: None,
            },
        },
    ));
    lso.body
        .push(Element::new("dd", Value::Amf3ObjectReference(ObjectId(0))));

    let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap();

    flash_lso::read::Reader::default().parse(&bytes).unwrap();
}
