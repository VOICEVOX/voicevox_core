#!/usr/bin/env bash
# !!! コードサイニング証明書を取り扱うので取り扱い注意 !!!

# macOS上で.p12証明書を使ってコード署名する

set -eu

if [ ! -v APPLE_P12_BASE64 ]; then # .p12証明書のbase64エンコードされた内容
    echo "APPLE_P12_BASE64が未定義です"
    exit 1
fi
if [ ! -v APPLE_P12_PASSWORD ]; then # .p12証明書のパスワード
    echo "APPLE_P12_PASSWORDが未定義です"
    exit 1
fi

if [ $# -ne 1 ]; then
    echo "引数の数が一致しません"
    exit 1
fi
target_file_glob="$1"

# .p12証明書のデコード
P12_PATH="$(mktemp -d)/codesign.p12"
echo "$APPLE_P12_BASE64" | base64 --decode > "$P12_PATH"

# 一時キーチェーンのセットアップ
KEYCHAIN_PATH="$(mktemp -d)/codesign.keychain-db"
KEYCHAIN_PASSWORD="$(uuidgen)"
security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

# Apple中間証明書のインポート
DEVELOPER_ID_G2_CA="$(mktemp)"
curl -fsSL -o "$DEVELOPER_ID_G2_CA" "https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer"
security import "$DEVELOPER_ID_G2_CA" -k "$KEYCHAIN_PATH"
rm "$DEVELOPER_ID_G2_CA"

# .p12証明書のインポート
security import "$P12_PATH" -k "$KEYCHAIN_PATH" -P "$APPLE_P12_PASSWORD" -T /usr/bin/codesign -A
security set-key-partition-list -S apple-tool:,apple: -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH" >/dev/null

ORIGINAL_KEYCHAINS=()
while IFS= read -r line; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line#\"}"
    line="${line%\"}"
    [ -n "$line" ] && ORIGINAL_KEYCHAINS+=("$line")
done < <(security list-keychains -d user)
security list-keychains -d user -s "$KEYCHAIN_PATH" "${ORIGINAL_KEYCHAINS[@]}"

IDENTITY=$(security find-identity -v -p codesigning "$KEYCHAIN_PATH" | awk 'match($0,/[0-9A-F]{40}/){print substr($0,RSTART,RLENGTH); exit}')
if [ -z "$IDENTITY" ]; then
    echo "署名用の有効なIdentityが見つかりません"
    exit 1
fi

# 証明書を破棄
cleanup() {
    security delete-keychain "$KEYCHAIN_PATH"
    rm -f "$P12_PATH"
}
trap cleanup EXIT

# 指定ファイルに署名する
function codesign_file() {
    TARGET="$1"
    codesign --force --sign "$IDENTITY" --timestamp "$TARGET"
}

# 指定ファイルがadhoc以外で署名されているか
function is_signed() {
    TARGET="$1"
    codesign -dv "$TARGET" 2>&1 | grep -q "^Signature=adhoc" && return 1
    codesign --verify "$TARGET" >/dev/null 2>&1 || return 1
}

# 署名されていなければ署名
# shellcheck disable=SC2012,SC2086
ls $target_file_glob | while read -r target_file; do
    if is_signed "$target_file"; then
        echo "署名済み: $target_file"
    else
        echo "署名開始: $target_file"
        codesign_file "$target_file"
    fi
done
