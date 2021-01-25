use core::fmt;
use flash_lso::errors::Error;
use flash_lso::read::Reader;
use flash_lso::write::Writer;
use nom::error::ErrorKind;
use nom::Err::Incomplete;
use nom::Needed;
// #[cfg(test)]
// use pretty_assertions::assert_eq;

/// Wrapper around Vec<u8> that makes `{:#?}` the same as `{:?}`
/// Used in `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyArray<'a>(pub &'a Vec<u8>);

/// Make diff to display string as single-line string
impl<'a> fmt::Debug for PrettyArray<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self.0))
    }
}

macro_rules! auto_test {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let parse_res = Reader::default().parse(data);

            if let Ok((unparsed_bytes, sol)) =  parse_res {
                println!("{:#?}", sol);

                let empty: Vec<u8> = vec![];
                if unparsed_bytes.len() > 0 {
                    assert_eq!(crate::PrettyArray(&empty), crate::PrettyArray(&unparsed_bytes[..unparsed_bytes.len().min(100)].to_vec()));
                }

                let bytes = flash_lso::write::write_to_bytes(&sol);

                assert_eq!(crate::PrettyArray(&bytes), crate::PrettyArray(&data.to_vec()), "library output != input");
            } else {
                println!("parse failed: {:?}", parse_res);
                assert_eq!(false, true)
            }
        }
        )*
    }
}

macro_rules! test_parse_only {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let parse_res = Reader::default().parse(data);

            if let Ok((unparsed_bytes, sol)) =  parse_res {
                println!("Parsed sol: {:?}", sol);
                let empty: Vec<u8> = vec![];
                if unparsed_bytes.len() > 0 {
                    assert_eq!(PrettyArray(&empty), PrettyArray(&unparsed_bytes[..100].to_vec()));
                }
            } else {
                println!("parse failed: {:?}", parse_res);
                assert_eq!(false, true)
            }
        }
        )*
    }
}

macro_rules! auto_test_flex {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[cfg(feature = "flex")]
        #[test]
        pub fn $name() {
            use flash_lso::extra::flex;
            use cookie_factory::gen;
            use flash_lso::write::Writer;

            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let mut des = Reader::default();
            flex::read::register_decoders(&mut des.amf3_decoder);
            let parse_res = des.parse(data);

            if let Ok((unparsed_bytes, sol)) =  parse_res {
                let empty: Vec<u8> = vec![];
                if unparsed_bytes.len() > 0 {
                    assert_eq!(PrettyArray(&empty), PrettyArray(&unparsed_bytes[..100].to_vec()));
                }

                let v = vec![];
                let mut s = Writer::default();
                flex::write::register_encoders(&mut s.amf3_encoder);
                let serialise = s.write_full(&sol);
                let (buffer, _size) = gen(serialise, v).unwrap();
                let bytes = buffer;

                let mut des2 = Reader::default();
                flex::read::register_decoders(&mut des2.amf3_decoder);
                let (_, sol2) = des2.parse(&bytes).expect("Unable to round trip");
                assert!(sol2 == sol);

                assert_eq!(PrettyArray(&bytes), PrettyArray(&data.to_vec()), "library output != input");
            } else {
                println!("parse failed: {:?}", parse_res);
                assert_eq!(false, true)
            }
        }
        )*
    }
}

macro_rules! should_fail {
    ($([$name: ident, $path: expr, $error: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let parse_res = Reader::default().parse(data);

            if let Err(x) = parse_res {
                assert_eq!(x, $error);
            } else {
                println!("error: {:?}", parse_res);
                assert_eq!("Correct error type", "Wrong error type");
            }
        }
        )*
    }
}

macro_rules! json_test {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[cfg(feature = "serde")]
        #[test]
        pub fn $name() {
            use serde_json;

            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let (_, parse_res) = Reader::default().parse(data).expect("Unable to parse file");
            let output_json = serde_json::to_string(&parse_res).expect("Unable to convert to json");


            let json_expected = include_str!(concat!("sol/", $path, ".json"));

            assert_eq!(json_expected.trim(), output_json);
        }
        )*
    }
}

macro_rules! json_test_flex {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[cfg(feature = "flex")]
        #[cfg(feature = "serde")]
        #[test]
        pub fn $name() {
            use serde_json;
            use flash_lso::extra::flex;

            let data = include_bytes!(concat!("sol/", $path, ".sol"));
            let mut des = Reader::default();
            flex::read::register_decoders(&mut des.amf3_decoder);
            let (_, parse_res) = des.parse(data).expect("Unable to parse file");
            let output_json = serde_json::to_string(&parse_res).expect("Unable to convert to json");

            let json_expected = include_str!(concat!("sol/", $path, ".json"));

            assert_eq!(json_expected.trim(), output_json);
        }
        )*
    }
}

json_test! {
    // AS2
    [json_as2_array, "AS2-Array-Demo"],
    [json_as2_boolean, "AS2-Boolean-Demo"],
    [json_as2_date, "AS2-Date-Demo"],
    [json_as2_demo, "AS2-Demo"],
    [json_as2_ecma_array, "AS2-ECMAArray-Demo"],
    [json_as2_integer, "AS2-Integer-Demo"],
    [json_as2_long_string, "AS2-LongString-Demo"],
    [json_as2_null, "AS2-Null-Demo"],
    [json_as2_number, "AS2-Number-Demo"],
    [json_as2_object, "AS2-Object-Demo"],
    [json_as2_string, "AS2-String-Demo"],
    [json_as2_typed_object, "AS2-TypedObject-Demo"],
    [json_as2_undefined, "AS2-Undefined-Demo"],
    [json_as2_xml, "AS2-XML-Demo"],
    // AS3
    [json_as3_number, "AS3-Number-Demo"],
    [json_as3_boolean, "AS3-Boolean-Demo"],
    [json_as3_string, "AS3-String-Demo"],
    [json_as3_object, "AS3-Object-Demo"],
    [json_as3_null, "AS3-Null-Demo"],
    [json_as3_undefined, "AS3-Undefined-Demo"],
    [json_as3_strict_array, "AS3-Array-Demo"],
    [json_as3_date, "AS3-Date-Demo"],
    [json_as3_xml, "AS3-XML-Demo"],
    [json_as3_xml_doc, "AS3-XMLDoc-Demo"],
    [json_as3_typed_object, "AS3-TypedObject-Demo"],
    [json_as3_integer, "AS3-Integer-Demo"],
    [json_as3_byte_array, "AS3-ByteArray-Demo"],
    [json_as3_vector_int, "AS3-VectorInt-Demo"],
    [json_as3_vector_unsigned_int, "AS3-VectorUint-Demo"],
    [json_as3_vector_number, "AS3-VectorNumber-Demo"],
    [json_as3_vector_object, "AS3-VectorObject-Demo"],
    [json_as3_vector_typed_object, "AS3-VectorTypedObject-Demo"],
    [json_as3_dictionary, "AS3-Dictionary-Demo"],
    [json_as3_demo, "AS3-Demo"],
    // Other
    [json_akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
    [json_areana_madness_game_two, "arenaMadnessGame2"],
    [json_canvas, "canvas"],
    [json_clarence_save_slot_1, "ClarenceSave_SLOT1"],
    [json_coc_8, "CoC_8"],
    [json_com_jeroenwijering, "com.jeroenwijering"],
    [json_cramjs, "cramjs"],
    [json_dolphin_show_1, "dolphin_show(1)"],
    [json_flagstaff_1, "flagstaff(1)"],
    [json_flagstaff_2, "flagstaff"],
    [json_flash_viewer, "flash.viewer"],
    [json_hiro_network_capping_cookie, "HIRO_NETWORK_CAPPING_COOKIE"],
    [json_jy1, "JY1"],
    [json_labrat_2, "Labrat2"],
    [json_mardek_v3_sg_1, "MARDEKv3__sg_1"],
    [json_media_player_user_settings, "mediaPlayerUserSettings"],
    [json_minimal, "Minimal"],
    [json_minimal_2, "Minimalv2"],
    [json_previous_video, "previousVideo"],
    [json_robokill, "robokill"],
    [json_settings, "settings"],
    [json_slot_1_party, "slot1_party"],
    [json_sound_data, "soundData"],
    [json_sound_data_level_0, "soundData_level0"],
    [json_space, "Space"],
    [json_string_test, "StringTest"],
    [json_time_display_config, "timeDisplayConfig"],
    [json_user_1, "user(1)"],
    [json_user, "user"],
    // Parse only
    [json_infectonator_survivors_76561198009932603, "InfectonatorSurvivors76561198009932603"],
    [json_slot_1, "slot1"],
    [json_party_1, "Party1"],
    [json_metadata_history, "MetadataHistory"]
}

json_test_flex! {
    [json_opp_detail_prefs, "oppDetailPrefs"]
}

// As2 / amf0
auto_test! {
    [as2_array, "AS2-Array-Demo"],
    [as2_boolean, "AS2-Boolean-Demo"],
    [as2_date, "AS2-Date-Demo"],
    [as2_demo, "AS2-Demo"],
    [as2_ecma_array, "AS2-ECMAArray-Demo"],
    [as2_integer, "AS2-Integer-Demo"],
    [as2_long_string, "AS2-LongString-Demo"],
    [as2_null, "AS2-Null-Demo"],
    [as2_number, "AS2-Number-Demo"],
    [as2_object, "AS2-Object-Demo"],
    [as2_string, "AS2-String-Demo"],
    [as2_typed_object, "AS2-TypedObject-Demo"],
    [as2_undefined, "AS2-Undefined-Demo"],
    [as2_xml, "AS2-XML-Demo"]
}

// As3 / amf3
#[cfg(feature = "amf3")]
pub mod amf3 {
    use crate::Reader;
    auto_test! {
        [as3_number, "AS3-Number-Demo"],
        [as3_boolean, "AS3-Boolean-Demo"],
        [as3_string, "AS3-String-Demo"],
        [as3_object, "AS3-Object-Demo"],
        [as3_null, "AS3-Null-Demo"],
        [as3_undefined, "AS3-Undefined-Demo"],
        [as3_strict_array, "AS3-Array-Demo"],
        [as3_date, "AS3-Date-Demo"],
        [as3_xml, "AS3-XML-Demo"],
        [as3_xml_doc, "AS3-XMLDoc-Demo"],
        [as3_typed_object, "AS3-TypedObject-Demo"],
        [as3_integer, "AS3-Integer-Demo"],
        [as3_byte_array, "AS3-ByteArray-Demo"],
        [as3_vector_int, "AS3-VectorInt-Demo"],
        [as3_vector_unsigned_int, "AS3-VectorUint-Demo"],
        [as3_vector_number, "AS3-VectorNumber-Demo"],
        [as3_vector_object, "AS3-VectorObject-Demo"],
        [as3_vector_typed_object, "AS3-VectorTypedObject-Demo"],
        [as3_dictionary, "AS3-Dictionary-Demo"]
    }
}

// Other tests, mixed
auto_test! {
    [akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
    [areana_madness_game_two, "arenaMadnessGame2"],
    [canvas, "canvas"],
    [clarence_save_slot_1, "ClarenceSave_SLOT1"],
    [coc_8, "CoC_8"],
    [com_jeroenwijering, "com.jeroenwijering"],
    [cramjs, "cramjs"],
    [dolphin_show_1, "dolphin_show(1)"],
    [flagstaff_1, "flagstaff(1)"],
    [flagstaff_2, "flagstaff"],
    [flash_viewer, "flash.viewer"],
    [hiro_network_capping_cookie, "HIRO_NETWORK_CAPPING_COOKIE"],
    [jy1, "JY1"],
    [labrat_2, "Labrat2"],
    [mardek_v3_sg_1, "MARDEKv3__sg_1"],
    [media_player_user_settings, "mediaPlayerUserSettings"],
    [minimal, "Minimal"],
    [minimal_2, "Minimalv2"],
    [previous_video, "previousVideo"],
    [robokill, "robokill"],
    [settings, "settings"],
    [slot_1_party, "slot1_party"],
    [sound_data, "soundData"],
    [sound_data_level_0, "soundData_level0"],
    [space, "Space"],
    [string_test, "StringTest"],
    [time_display_config, "timeDisplayConfig"],
    [user_1, "user(1)"],
    [user, "user"]
}

// Samples that can be parsed but not written
test_parse_only! {
    [infectonator_survivors_76561198009932603, "InfectonatorSurvivors76561198009932603"],
    [slot_1, "slot1"],
    [party_1, "Party1"],

    // External classes
    [metadata_history, "MetadataHistory"],

    [as3_demo, "AS3-Demo"]
}

auto_test_flex! {
    [opp_detail_prefs, "oppDetailPrefs"]
}

should_fail! {
    // Corrupt/invalid file
    [two, "2",  nom::Err::Error(Error::Nom(vec![17, 112, 99, 95, 112, 97, 114, 116, 121, 10, 130, 51, 21, 80, 97, 114, 116, 121, 65, 108, 105, 97, 115, 0, 13, 98, 97, 116, 116, 108, 101, 2, 0].as_slice(), ErrorKind::Tag))],
    // OOB read
    [zero_four, "00000004", nom::Err::Error(Error::Nom(vec![0, 255, 0, 0, 0, 86, 0, 84, 47, 117, 112, 108, 111, 97, 100, 115, 46, 117, 110, 103, 114, 111, 117, 110, 100, 101, 100, 46, 110, 101, 116, 47, 53, 57, 50, 48, 48, 48, 47, 53, 57, 50, 52, 55, 51, 95, 77, 97, 100, 110, 101, 115, 115, 71, 97, 109, 101, 95, 85, 76, 84, 73, 77, 65, 84, 69, 46, 115, 119, 102, 47, 97, 114, 101, 110, 97, 77, 97, 100, 110, 101, 115, 115, 71, 97, 109, 101, 50, 46, 115, 111, 108].as_slice(), ErrorKind::Eof))]
}
