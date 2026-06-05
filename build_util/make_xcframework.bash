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
if [ ! -v MACOS_ARM64_PATH ]; then # MACOS_ARM64用のモジュールのディレクトリ
    echo "MACOS_ARM64_PATHが未定義です"
    exit 1
fi
if [ ! -v MACOS_X64_PATH ]; then # MACOS_X64用のモジュールのディレクトリ
    echo "MACOS_X64_PATHが未定義です"
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
platforms=("ios" "sim" "macos")
artifacts=("${IOS_AARCH64_PATH}" "${IOS_AARCH64_SIM_PATH}" "${MACOS_ARM64_PATH}")

declare -a link_onnxruntime_platform=()
# ONNXRUNTIMEのリンクが必要なプラットフォームを確認
for i in "${!platforms[@]}"; do
    platform="${platforms[$i]}"
    artifact="${artifacts[$i]}"
    if grep -q -e '^#define VOICEVOX_LINK_ONNXRUNTIME$' -e '^//#define VOICEVOX_LOAD_ONNXRUNTIME$' "${artifact}/include/voicevox_core.h"; then
        link_onnxruntime_platform+=("${platform}")
    fi
done

for i in "${!platforms[@]}"; do
    platform="${platforms[$i]}"
    artifact="${artifacts[$i]}"
    echo "* Copy Framework-${platform} template"
    mkdir -p "Framework-${platform}/voicevox_core.framework/Headers"
    cp -vr "crates/voicevox_core_c_api/xcframework/Frameworks/${platform}/" "Framework-${platform}/"
    cp -v "${artifact}/include/voicevox_core.h" \
        "Framework-${platform}/voicevox_core.framework/Headers/voicevox_core.h"
done

echo "* Create dylib"
# iosはdylibをコピー
cp -v "${IOS_AARCH64_PATH}/lib/libvoicevox_core.dylib" \
    "Framework-ios/voicevox_core.framework/voicevox_core"

# simはx86_64とarrch64を合わせてdylib作成
lipo -create "${IOS_X86_64_PATH}/lib/libvoicevox_core.dylib" \
    "${IOS_AARCH64_SIM_PATH}/lib/libvoicevox_core.dylib" \
    -output "Framework-sim/voicevox_core.framework/voicevox_core"

# macosはx64とarrch64を合わせてdylib作成
lipo -create "${MACOS_X64_PATH}/lib/libvoicevox_core.dylib" \
    "${MACOS_ARM64_PATH}/lib/libvoicevox_core.dylib" \
    -output "Framework-macos/voicevox_core.framework/voicevox_core"

for platform in "${platforms[@]}"; do
    echo "* Change ${platform} @rpath"
    # 自身への@rpathを変更
    install_name_tool -id "@rpath/voicevox_core.framework/voicevox_core" \
        "Framework-${platform}/voicevox_core.framework/voicevox_core"
done

for platform in "${link_onnxruntime_platform[@]}"; do
    echo "* Change ${platform} onnxruntime @rpath"
    # onnxruntimeへの@rpathを、voicevox_onnxruntimeのXCFrameworkに変更
    install_name_tool -change "@rpath/$dylib_string" \
        "@rpath/voicevox_onnxruntime.framework/voicevox_onnxruntime" \
        "Framework-${platform}/voicevox_core.framework/voicevox_core"
done

echo "* Restructure macOS Framework to versioned format"
# macOSのFrameworkはxcodebuild -create-xcframeworkに渡す際にVersioned directory構造が必要
framework_dir="Framework-macos/voicevox_core.framework"
mkdir -p "${framework_dir}/Versions/A/Resources"
mkdir -p "${framework_dir}/Versions/A/Headers"
mkdir -p "${framework_dir}/Versions/A/Modules"
mv "${framework_dir}/voicevox_core" "${framework_dir}/Versions/A/voicevox_core"
mv "${framework_dir}/Headers/voicevox_core.h" "${framework_dir}/Versions/A/Headers/voicevox_core.h"
mv "${framework_dir}/Info.plist" "${framework_dir}/Versions/A/Resources/Info.plist"
mv "${framework_dir}/Modules/module.modulemap" "${framework_dir}/Versions/A/Modules/module.modulemap"
rm -rf "${framework_dir}/Headers" "${framework_dir}/Modules"
ln -sf A "${framework_dir}/Versions/Current"
ln -sf Versions/Current/voicevox_core "${framework_dir}/voicevox_core"
ln -sf Versions/Current/Resources "${framework_dir}/Resources"
ln -sf Versions/Current/Headers "${framework_dir}/Headers"
ln -sf Versions/Current/Modules "${framework_dir}/Modules"

echo "* Create XCFramework"
mkdir -p "${OUTPUT_ASSET_PATH}"
xcodebuild -create-xcframework \
    -framework "Framework-sim/voicevox_core.framework" \
    -framework "Framework-ios/voicevox_core.framework" \
    -framework "Framework-macos/voicevox_core.framework" \
    -output "${OUTPUT_ASSET_PATH}/voicevox_core.xcframework"
