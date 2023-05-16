import numpy as np
import os
from pathlib import Path
import toml

data_dir = Path(os.path.dirname(os.path.abspath(__file__)))

example_f0_length = 69
example_phoneme_size = 45

example_f0 = np.load(data_dir / "example_f0.npy")
example_phoneme = np.load(data_dir / "example_phoneme.npy")

snapshots = toml.load(
    data_dir.parent.parent.parent.parent
    / "voicevox_core_c_api"
    / "tests"
    / "e2e"
    / "snapshots.toml"
)

example_duration = np.array(
    snapshots["compatible_engine"]["yukarin_s_forward"], dtype=np.float32
)
example_intonation = np.array(
    snapshots["compatible_engine"]["yukarin_sa_forward"], dtype=np.float32
)
