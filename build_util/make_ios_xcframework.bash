#!/usr/bin/env bash
set -eu

if [ ! -v IOS_X86_64_PATH ]; then # X86_64用のモジュールのディレクトリ(simulator)
    echo "IOS_X86_64_PATHが未定義です"
    exit 1
fi
if [ ! -v IOS_AARCH64_SIM_PATH ]; then # AARCH64_SIM用のモジュールのディレクトリ(simulator)
    echo "IOS_AARCH64_SIM_PATHが未定義です"
    exit 1
fi
if [ ! -v IOS_AARCH64_PATH ]; then # AARCH64用のモジュールのディレクトリ(実機)
    echo "IOS_AARCH64_PATHが未定義です"
    exit 1
fi
if [ ! -v OUTPUT_ASSET_PATH ]; then # 出力するASSETのディレクトリ
    echo "OUTPUT_ASSET_PATHが未定義です"
    exit 1
fi

echo "* Get original onnxruntime file name from rpath"
output=$(otool -L "${IOS_AARCH64_PATH}/lib/libvoicevox_core.dylib")
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

echo "* Copy Framework template"
arches=("aarch64" "sim")
artifacts=("${IOS_AARCH64_PATH}" "${IOS_AARCH64_SIM_PATH}")
for i in "${!arches[@]}"; do
    arch="${arches[$i]}"
    artifact="${artifacts[$i]}"
    echo "* Copy Framework-${arch} template"
    mkdir -p "Framework-${arch}/voicevox_core.framework/Headers"
    cp -vr "crates/voicevox_core_c_api/xcframework/Frameworks/${arch}/" "Framework-${arch}/"
    cp -v "${artifact}/include/voicevox_core.h" \
        "Framework-${arch}/voicevox_core.framework/Headers/voicevox_core.h"
done

echo "* Create dylib"
# aarch64はdylibをコピー
cp -v "${IOS_AARCH64_PATH}/lib/libvoicevox_core.dylib" \
    "Framework-aarch64/voicevox_core.framework/voicevox_core"

# simはx86_64とarrch64を合わせてdylib作成
lipo -create "${IOS_X86_64_PATH}/lib/libvoicevox_core.dylib" \
    "${IOS_AARCH64_SIM_PATH}/lib/libvoicevox_core.dylib" \
    -output "Framework-sim/voicevox_core.framework/voicevox_core"

for arch in "${arches[@]}"; do
    echo "* Change ${arch} @rpath"
    # 自身への@rpathを変更
    install_name_tool -id "@rpath/voicevox_core.framework/voicevox_core" \
        "Framework-${arch}/voicevox_core.framework/voicevox_core"

    # onnxruntimeへの@rpathを、voicevox_onnxruntimeのXCFrameworkに変更
    install_name_tool -change "@rpath/$dylib_string" \
        "@rpath/voicevox_onnxruntime.framework/voicevox_onnxruntime" \
        "Framework-${arch}/voicevox_core.framework/voicevox_core"
done

echo "* Create XCFramework"
mkdir -p "${OUTPUT_ASSET_PATH}"
xcodebuild -create-xcframework \
    -framework "Framework-sim/voicevox_core.framework" \
    -framework "Framework-aarch64/voicevox_core.framework" \
    -output "${OUTPUT_ASSET_PATH}/voicevox_core.xcframework"
