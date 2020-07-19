use core::fmt;
use flash_lso::encoder;
use flash_lso::LSODeserializer;
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::fs::File;
use std::io::Read;

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
            let mut x = File::open(concat!("tests/sol/", $path, ".sol")).expect("Couldn't open file");
            let mut data = Vec::new();
            let _ = x.read_to_end(&mut data).expect("Unable to read file");
            let parse_res = LSODeserializer::default().parse_full(&data);

            if let Ok((unparsed_bytes, sol)) =  parse_res {

            println!("{:#?}", sol);
            // println!("Unparsed bytes: {:?}", unparsed_bytes);

            let empty: Vec<u8> = vec![];
            if unparsed_bytes.len() > 0 {
                assert_eq!(PrettyArray(&empty), PrettyArray(&unparsed_bytes[..100].to_vec()));
            }

            // let bytes = encoder::write_to_bytes(&sol);
            // assert_eq!(PrettyArray(&bytes), PrettyArray(&data), "library output != input");
            } else {
                println!("Input: {:?}", data);
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
            let mut x = File::open(concat!("tests/sol/", $path, ".sol")).expect("Couldn't open file");
            let mut data = Vec::new();
            let _ = x.read_to_end(&mut data).expect("Unable to read file");
            let parse_res = LSODeserializer::default().parse_full(&data);

            if let Ok((unparsed_bytes, sol)) =  parse_res {
                let empty: Vec<u8> = vec![];
                if unparsed_bytes.len() > 0 {
                    assert_eq!(PrettyArray(&empty), PrettyArray(&unparsed_bytes[..100].to_vec()));
                }
            } else {
                println!("Input: {:?}", data);
                println!("parse failed: {:?}", parse_res);
                assert_eq!(false, true)
            }
        }
        )*
    }
}

use nom::error::make_error;
use nom::Err::Error;
use nom::Err::Incomplete;

macro_rules! should_fail {
    ($([$name: ident, $path: expr, $error_enum: ident, $error: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let mut x = File::open(concat!("tests/sol/", $path, ".sol")).expect("Couldn't open file");
            let mut data = Vec::new();
            let _ = x.read_to_end(&mut data).expect("Unable to read file");
            let parse_res = LSODeserializer::default().parse_full(&data);

            if let Err(x) = parse_res {
                assert_eq!(x, $error_enum($error));
            } else {
                println!("error: {:?}", parse_res);
                assert_eq!("Correct error type", "Wrong error type");
            }
        }
        )*
    }
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

// Other tests, mixed
auto_test! {
    // [two, "2"],
    // [zero_four, "00000004"],
    [akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
    // [areana_madness_game_two, "arenaMadnessGame2"]
    [canvas, "canvas"],
    // [clarence_save_slot_1, "ClarenceSave_SLOT1"],
    // [CoC_8, "CoC_8"],
    [com_jeroenwijering, "com.jeroenwijering"],
    [cramjs, "cramjs"],
    [dolphin_show_1, "dolphin_show(1)"],
    [flagstaff_1, "flagstaff(1)"],
    // [flagstaff, "flagstaff"],
    [flash_viewer, "flash.viewer"],
    // [hiro_network_capping_cookie, "HIRO_NETWORK_CAPPING_COOKIE"],
    // [infectonator_survivors_76561198009932603, "InfectonatorSurvivors76561198009932603"],
    // [jy1, "JY1"],
    // [labrat_2, "Labrat2"],
    // [mardek_v3_sg_1, "MARDEKv3__sg_1"],
    [media_player_user_settings, "mediaPlayerUserSettings"],
    // [metadata_history, "MetadataHistory"],
    [minimal, "Minimal"],
    [minimal_2, "Minimalv2"],
    // [opp_detail_prefs, "oppDetailPrefs"],
    // [party_1, "Party1"],
    [previous_video, "previousVideo"],
    // [robokill, "robokill"],
    [settings, "settings"],
    // [slot_1, "slot1"],
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
    [clarence_save_slot_1, "ClarenceSave_SLOT1"],
    [slot_1_asf, "slot1"], // malloc error
    [mardek_v3_sg_1, "MARDEKv3__sg_1"], // memory error - amf3? maybe
    [jy1, "JY1"], // Malloc too big - amf0
    [CoC_8, "CoC_8"], // Gets SIGKILLED? memory error
    [robokill, "robokill"] // Invalid write
}


// Other tests, completly failing
auto_test! {
    // [flagstaff, "flagstaff"] // TODO: external class, probably wont parse
    // [metadata_history, "MetadataHistory"] // External class, probably wont parse
    // [opp_detail_prefs, "oppDetailPrefs"] //TODO: uses flex, probably wont parse for a while

    // [areana_madness_game_two, "arenaMadnessGame2"], //Huge
    // [labrat_2, "Labrat2"] // huge
        //     [party_1, "Party1"] // huge
}
//24

use nom::error::ErrorKind::Tag;
use nom::Needed;
should_fail! {
    // Corrupt/invalid file
    [two, "2", Error, (vec![17, 112, 99, 95, 112, 97, 114, 116, 121, 10, 130, 51, 21, 80, 97, 114, 116, 121, 65, 108, 105, 97, 115, 0, 13, 98, 97, 116, 116, 108, 101, 2, 0].as_slice(), Tag)],
    // OOB read
    [zero_four, "00000004", Incomplete, Needed::Size(255)]
}
