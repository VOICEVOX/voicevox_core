#!/usr/bin/env bash
set -eu

echo "* Get original onnxruntime file name from rpath"
output=$(otool -L "artifact/voicevox_core-aarch64-apple-ios/libvoicevox_core.dylib")
matched_line=$(echo "$output" | grep "@rpath" | grep "libonnxruntime")
if [[ $matched_line ]]; then
    if [[ $matched_line =~ (@rpath/([^ ]+\.dylib)) ]]; then
        dylib_string=${BASH_REMATCH[2]}
    else
        echo "Expected pattern not found in the matched line"
        echo "$output"
        exit 1
    fi
else
    echo "No line containing '@rpath' and 'libonnxruntime' found"
    echo "$output"
    exit 1
fi
echo "Original onnx dylib file name: $dylib_string"

echo "* Create aarch64 Framework"
mkdir -p "Framework-aarch64/voicevox_core.framework/Headers"
cp -vr "crates/voicevox_core_c_api/xcframework/Frameworks/aarch64/" \
    "Framework-aarch64/"
cp -v "artifact/voicevox_core-aarch64-apple-ios/voicevox_core.h" \
    "Framework-aarch64/voicevox_core.framework/Headers/voicevox_core.h"
cp -v "artifact/voicevox_core-aarch64-apple-ios/libvoicevox_core.dylib" \
    "Framework-aarch64/voicevox_core.framework/voicevox_core"

echo "* Change aarch64 @rpath"
# 自身への@rpathを変更
install_name_tool -id "@rpath/voicevox_core.framework/voicevox_core" \
    "Framework-aarch64/voicevox_core.framework/voicevox_core"
# 依存ライブラリを調べると@rpath/libonnxruntime.A.BB.C.dylibの文字列があるので、
# これを@rpath/onnxruntime.framework/onnxruntimeに変更したい。
# onnxruntimeへの@rpathを変更
install_name_tool -change "@rpath/$dylib_string" \
    "@rpath/onnxruntime.framework/onnxruntime" \
    "Framework-aarch64/voicevox_core.framework/voicevox_core"

echo "* Create sim Framework"
mkdir -p "Framework-sim/voicevox_core.framework/Headers"
cp -vr "crates/voicevox_core_c_api/xcframework/Frameworks/sim/" "Framework-sim/"
cp -v "artifact/voicevox_core-aarch64-apple-ios/voicevox_core.h" \
    "Framework-sim/voicevox_core.framework/Headers/voicevox_core.h"
lipo -create "artifact/voicevox_core-x86_64-apple-ios/libvoicevox_core.dylib" \
    "artifact/voicevox_core-aarch64-apple-ios-sim/libvoicevox_core.dylib" \
    -output "Framework-sim/voicevox_core.framework/voicevox_core"

echo "* Change sim @rpath"   
# 自身への@rpathを変更
install_name_tool -id "@rpath/voicevox_core.framework/voicevox_core" \
    "Framework-sim/voicevox_core.framework/voicevox_core"
# onnxruntimeへの@rpathを変更
install_name_tool -change "@rpath/$dylib_string" \
    "@rpath/onnxruntime.framework/onnxruntime" \
    "Framework-sim/voicevox_core.framework/voicevox_core"

echo "* Create XCFramework"
mkdir -p "artifact/${ASSET_NAME}"
xcodebuild -create-xcframework \
    -framework "Framework-sim/voicevox_core.framework" \
    -framework "Framework-aarch64/voicevox_core.framework" \
    -output "artifact/${ASSET_NAME}/voicevox_core.xcframework"
