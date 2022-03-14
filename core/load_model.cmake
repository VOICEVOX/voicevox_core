# テキストのu8Rを頭につけて""で囲うことで、改行のあるutf8ファイルを#includeできるようにする
function(make_includable_text input_file)
    set(content "u8R\"(\n${content})\"")
    get_filename_component(output_file ${input_file} NAME)
    file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/${output_file}" "${content}")
endfunction(make_includable_text)


# # バイナリファイルを1バイトごとにカンマ区切りでテキスト出力する
# function(binary_to_hex_text bin_file)
#     file(READ ${bin_file} string_hex HEX)
#     string(REGEX MATCHALL "([A-Fa-f0-9][A-Fa-f0-9])" separated_hex ${string_hex})
#     list(JOIN separated_hex ", 0x" formatted_hex)
#     string(PREPEND formatted_hex "0x")
#     get_filename_component(filename ${bin_file} NAME)
#     file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/${filename}.txt" "${formatted_hex}")
# endfunction(binary_to_hex_text)


