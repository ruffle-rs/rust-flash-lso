use std::fs::File;
use std::io::Read;
use flash_lso::LSODeserializer;
use flash_lso::encoder;

macro_rules! auto_test {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let mut x = File::open(concat!("tests/sol/", $path, ".sol")).expect("Couldn't open file");
            let mut data = Vec::new();
            let _ = x.read_to_end(&mut data).expect("Unable to read file");
            let (unparsed_bytes, sol) = LSODeserializer::default().parse_full(&data).expect("Failed to parse lso file");

            println!("{:?}", sol);
            println!("Unparsed bytes: {:?}", unparsed_bytes);

            let bytes = encoder::write_to_bytes(&sol);
            assert_eq!(bytes, data)
        }
        )*
    }
}

auto_test! {
    [as2_array, "AS2-Array-Demo"],
    [as2_boolean, "AS2-Boolean-Demo"],
    [as2_date, "AS2-Date-Demo"],
    // [as2_demo, "AS2-Demo"]
    [as2_ecma_array, "AS2-ECMAArray-Demo"],
    [as2_integer, "AS2-Integer-Demo"],
    [as2_long_string, "AS2-LongString-Demo"],
    [as2_null, "AS2-Null-Demo"],
    [as2_number, "AS2-Number-Demo"],
    // [as2_object, "AS2-Object-Demo"],
    [as2_string, "AS2-String-Demo"],
    // [as2_typed_object, "AS2-TypedObject-Demo"]
    [as2_undefined, "AS2-Undefined-Demo"]
    // [as2_xml, "AS2-XML-Demo"]
}
