#!/bin/bash


cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/self-referential-object.amf > ./flash-lso/tests/amf/self-referential-object.json
cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/LearnToFly3.profileData.saveString.amf > ./flash-lso/tests/amf/LearnToFly3.profileData.saveString.json
cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/object-with-vec-obj-child-referencing-parent.amf > ./flash-lso/tests/amf/object-with-vec-obj-child-referencing-parent.json
cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/self-referential-dict.amf > ./flash-lso/tests/amf/self-referential-dict.json
cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/self-referential-array.amf > ./flash-lso/tests/amf/self-referential-array.json
cargo r --release --package lso-to-json -- object-amf3 ./flash-lso/tests/amf/self-referential-vec-object.amf > ./flash-lso/tests/amf/self-referential-vec-object.json


find ./flash-lso/tests/sol/ -type f -name "*.sol" | while read -r file; do
    output="${file%.sol}.json"
    cargo r --release --package lso-to-json -- file "$file" > "$output"
done
rm ./flash-lso/tests/sol/2.json
rm ./flash-lso/tests/sol/00000004.json
rm ./flash-lso/tests/sol/AS2-Date-Demo.sol.json
rm ./flash-lso/tests/sol/AkamaiEnterprisePlayer.userData.sol.json