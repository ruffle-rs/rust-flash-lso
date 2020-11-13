#![feature(test)]

extern crate test;

use flash_lso::LSODeserializer;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

macro_rules! auto_bench {
        ($([$name: ident, $path: expr]),*) => {
            fn criterion_benchmark(c: &mut Criterion) {
                $(
                    c.bench_function(concat!("parse_", $path), |b| {
                        let input_bytes = include_bytes!(concat!("../tests/sol/", $path, ".sol"));
                        b.iter(|| {
                            black_box(LSODeserializer::default().parse_full(input_bytes).unwrap());
                        })
                    });
                )*
            }
        }
    }

auto_bench! {
        // AS2
        [bench_as2_array, "AS2-Array-Demo"],
        [bench_as2_boolean, "AS2-Boolean-Demo"],
        [bench_as2_date, "AS2-Date-Demo"],
        [bench_as2_demo, "AS2-Demo"],
        [bench_as2_ecma_array, "AS2-ECMAArray-Demo"],
        [bench_as2_integer, "AS2-Integer-Demo"],
        [bench_as2_long_string, "AS2-LongString-Demo"],
        [bench_as2_null, "AS2-Null-Demo"],
        [bench_as2_number, "AS2-Number-Demo"],
        [bench_as2_object, "AS2-Object-Demo"],
        [bench_as2_string, "AS2-String-Demo"],
        [bench_as2_typed_object, "AS2-TypedObject-Demo"],
        [bench_as2_undefined, "AS2-Undefined-Demo"],
        [bench_as2_xml, "AS2-XML-Demo"],
        // AS3
        [bench_as3_number, "AS3-Number-Demo"],
        [bench_as3_boolean, "AS3-Boolean-Demo"],
        [bench_as3_string, "AS3-String-Demo"],
        [bench_as3_object, "AS3-Object-Demo"],
        [bench_as3_null, "AS3-Null-Demo"],
        [bench_as3_undefined, "AS3-Undefined-Demo"],
        [bench_as3_strict_array, "AS3-Array-Demo"],
        [bench_as3_date, "AS3-Date-Demo"],
        [bench_as3_xml, "AS3-XML-Demo"],
        [bench_as3_xml_doc, "AS3-XMLDoc-Demo"],
        [bench_as3_typed_object, "AS3-TypedObject-Demo"],
        [bench_as3_integer, "AS3-Integer-Demo"],
        [bench_as3_byte_array, "AS3-ByteArray-Demo"],
        [bench_as3_vector_int, "AS3-VectorInt-Demo"],
        [bench_as3_vector_unsigned_int, "AS3-VectorUint-Demo"],
        [bench_as3_vector_number, "AS3-VectorNumber-Demo"],
        [bench_as3_vector_object, "AS3-VectorObject-Demo"],
        [bench_as3_vector_typed_object, "AS3-VectorTypedObject-Demo"],
        [bench_as3_dictionary, "AS3-Dictionary-Demo"],
        [bench_as3_demo, "AS3-Demo"],
        // Other
        [bench_akamai_enterprise_player, "AkamaiEnterprisePlayer.userData"],
        [bench_areana_madness_game_two, "arenaMadnessGame2"],
        [bench_canvas, "canvas"],
        [bench_clarence_save_slot_1, "ClarenceSave_SLOT1"],
        [bench_coc_8, "CoC_8"],
        [bench_com_jeroenwijering, "com.jeroenwijering"],
        [bench_cramjs, "cramjs"],
        [bench_dolphin_show_1, "dolphin_show(1)"],
        [bench_flagstaff_1, "flagstaff(1)"],
        [bench_flagstaff_2, "flagstaff"],
        [bench_flash_viewer, "flash.viewer"],
        [bench_hiro_network_capping_cookie, "HIRO_NETWORK_CAPPING_COOKIE"],
        [bench_jy1, "JY1"],
        [bench_labrat_2, "Labrat2"],
        [bench_mardek_v3_sg_1, "MARDEKv3__sg_1"],
        [bench_media_player_user_settings, "mediaPlayerUserSettings"],
        [bench_minimal, "Minimal"],
        [bench_minimal_2, "Minimalv2"],
        [bench_previous_video, "previousVideo"],
        [bench_robokill, "robokill"],
        [bench_settings, "settings"],
        [bench_slot_1_party, "slot1_party"],
        [bench_sound_data, "soundData"],
        [bench_sound_data_level_0, "soundData_level0"],
        [bench_space, "Space"],
        [bench_string_test, "StringTest"],
        [bench_time_display_config, "timeDisplayConfig"],
        [bench_user_1, "user(1)"],
        [bench_user, "user"],
        // Parse only
        [bench_infectonator_survivors_76561198009932603, "InfectonatorSurvivors76561198009932603"],
        [bench_slot_1, "slot1"],
        [bench_party_1, "Party1"],
        [bench_metadata_history, "MetadataHistory"]
}
