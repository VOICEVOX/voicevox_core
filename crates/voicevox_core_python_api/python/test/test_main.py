import voicevox_core

def test_initialize(open_jtalk_dict_dir: str):
    voicevox_core.VoicevoxCore(open_jtalk_dict_dir=open_jtalk_dict_dir)
