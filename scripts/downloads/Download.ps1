#!/usr/bin/env pwsh

<#
	.DESCRIPTION
	voicevox_coreをダウンロードするためのスクリプト
#>


Param(
	[Parameter()]
	[String]
	[Alias("o")]
	# 出力先の指定
	$Output = "./voicevox_core",
	[Parameter()]
	[String]
	[Alias("v")]
	# ダウンロードするvoicevox_coreのバージョンの指定
	$Version = "latest",
	[Parameter()]
	[ValidateSet("cpu","cuda","directml")]
	[string]
	# ダウンロードするTypeを指定する(cpu,cuda,directmlを指定可能)
	$Type = "cpu",
	[Parameter()]
	[bool]
	# ダウンロードするライブラリを最小限にするように指定
	$Min = $False
)

$VoicevoxCoreRepositoryBaseUrl="https://github.com/VOICEVOX/voicevox_core"
$OpenJtalkDictUrl="https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
$OpenJtalkDictDirName="open_jtalk_dic_utf_8-1.11"

Function Voicevox-Core-Releases-Url($Os,$CpuArch,$Type,$Version){
	"${VoicevoxCoreRepositoryBaseUrl}/releases/download/${Version}/voicevox_core-${Os}-${CpuArch}-${type}-${Version}.zip"
}

Function Latest-Version($BaseUrl){
	$GetLatestUrl = "${BaseUrl}/releases/tag"
	try{Invoke-WebRequest "${BaseUrl}/releases/latest" -Method "Get" -MaximumRedirection 0 }catch{$_.Exception.Response.Headers.Location -replace "${GetLatestUrl}/","" }
}

Function Latest-Voicevox-Core-Version(){
	Latest-Version $VoicevoxCoreRepositoryBaseUrl
}

Function Target-Os(){
	"windows"
}

Function Target-Arch(){
	# TODO: cpu archtectureの判定を実装する
	"x64"
}

Function Download-and-Extract($Target,$Url,$ExtractDir,$ArchiveFormat){
	$TmpPath=New-TemporaryFile
	
	if ( [string]::IsNullOrEmpty($ArchiveFormat) ){
		if ( $Url.EndsWith(".tar.gz") ){
			$ArchiveFormat="tar.gz"
		} else {
			$ArchiveFormat="zip"
		}
	}

	echo "${Target}を${Url}からファイルをダウンロードします..."
	Invoke-WebRequest "${Url}" -OutFile "${TmpPath}"
	echo "${Target}をダウンロード完了,${ArchiveFormat}形式で${ExtractDir}に解凍します..."
	If( $ArchiveFormat -eq "zip" ){
		$Zip=[System.IO.Compression.ZipFile]::OpenRead("${TmpPath}")
		$Zip.Entries.ForEach{
			if ([string]::IsNullOrEmpty($_.Name)){
				return
			}
			$NewFile=[IO.FileInfo]($ExtractDir,$_.Name -join "/")
			$NewFile.Directory.Create()
			[System.IO.Compression.ZipFileExtensions]::ExtractToFile($_,$NewFile)
		}
		$Zip.Dispose()
	}ElseIf( $ArchiveFormat -eq "tar.gz" ){
		mkdir -p "$ExtractDir"
		tar --strip-components 1 -xvzf "$TmpPath" -C "$ExtractDir"
	}
	echo "${Target}のファイルを展開完了しました"
}

$Os=Target-Os
$CpuArch=Target-Arch
$OpenJtalkOutput="${Output}/${OpenJtalkDictDirName}"

if ( $Version -eq "latest" ){
	$Version=Latest-Voicevox-Core-Version
}

echo "対象OS:$Os"
echo "対象CPUアーキテクチャ:$cpu_arch"
echo "ダウンロードvoicevox_coreバージョン:$version"
echo "ダウンロードアーティファクトタイプ:$type"

$VoicevoxCoreUrl=Voicevox-Core-Releases-Url "$Os" "$CpuArch" "$Type" "$Version"

Download-and-Extract "voicevox_core" "$VoicevoxCoreUrl" "$Output"

if ( -not $Min ){
	Download-and-Extract "open_jtalk" "$OpenJtalkDictUrl" "$OpenJtalkOutput"
}

