use flash_lso::encoder;
use flash_lso::LSODeserializer;
use std::fs::File;
use std::io::Read;

macro_rules! auto_test {
    ($([$name: ident, $path: expr]),*) => {
        $(
        #[test]
        pub fn $name() {
            let mut x = File::open(concat!("tests/sol/", $path, ".sol")).expect("Couldn't open file");
            let mut data = Vec::new();
            let _ = x.read_to_end(&mut data).expect("Unable to read file");
            let (unparsed_bytes, sol) = LSODeserializer::default().parse_full(&data).expect("Failed to parse lso file");

            // println!("{:#?}", sol);
            // println!("Unparsed bytes: {:?}", unparsed_bytes);

            let bytes = encoder::write_to_bytes(&sol);
            assert_eq!(bytes, data, "library output != input")
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
    // [akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
    // [areana_madness_game_two, "arenaMadnessGame2"]
    [canvas, "canvas"],
    // [clarence_save_slot_1, "ClarenceSave_SLOT1"],
    // [CoC_8, "CoC_8"],
    // [com_jeroenwijering, "com.jeroenwijering"]
    [cramjs, "cramjs"],
    // [dolphin_show_1, "dolphin_show(1)"],
    // [flagstaff_1, "flagstaff(1)"],
    // [flagstaff, "flagstaff"],
    // [flash_viewer, "flash.viewer"],
    [hiro_network_capping_cookie, "HIRO_NETWORK_CAPPING_COOKIE"],
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
    // [previous_video, "previousVideo"],
    // [robokill, "robokill"],
    [settings, "settings"],
    // [slot_1, "slot1"],
    // [slow_1_party, "slot1_party"],
    [sound_data, "soundData"],
    [sound_data_level_0, "soundData_level0"],
    [space, "Space"],
    // [string_test, "StringTest"],


    [time_display_config, "timeDisplayConfig"]
    // [user_1, "user(1)"],
    // [user, "user"]
}

// Other tests, failing
// auto_test! {
//     [two, "2"],
//     [zero_four, "00000004"],
//     [akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
//     [areana_madness_game_two, "arenaMadnessGame2"],
//     [clarence_save_slot_1, "ClarenceSave_SLOT1"],
//     [CoC_8, "CoC_8"],
//     [com_jeroenwijering, "com.jeroenwijering"],
//     [dolphin_show_1, "dolphin_show(1)"],
//     [flagstaff_1, "flagstaff(1)"],
//     [flagstaff, "flagstaff"],
//     [flash_viewer, "flash.viewer"],
//     [infectonator_survivors_76561198009932603, "InfectonatorSurvivors76561198009932603"],
//     [jy1, "JY1"],
//     [labrat_2, "Labrat2"],
//     [mardek_v3_sg_1, "MARDEKv3__sg_1"],
//     [metadata_history, "MetadataHistory"],
//     [opp_detail_prefs, "oppDetailPrefs"],
//     [party_1, "Party1"],
//     [previous_video, "previousVideo"],
//     [robokill, "robokill"],
//     [slot_1, "slot1"],
//     [slow_1_party, "slot1_party"],
//     [string_test, "StringTest"],
//     [user_1, "user(1)"],
//     [user, "user"]
// }
//24


