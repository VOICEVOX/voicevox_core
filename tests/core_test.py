import json
import os
import unittest

import numpy as np

import core

root_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../model')

class TestCore(unittest.TestCase):
    def test_initialize_cpu(self):
        core.initialize(root_dir, False)
        core.finalize()

    def test_invalid_initialize_path(self):
        with self.assertRaises(Exception):
            core.initialize(" ", False)

    def test_invalid_speaker_id(self):
        core.initialize(root_dir, False)
        nil = np.array([], np.int64)
        nil2 = np.array([[]], np.int64)
        fnil2 = np.array([[]], np.float32)
        neg = np.array([-1], np.int64)
        with self.assertRaisesRegex(Exception, "Unknown style ID: -1"):
            core.yukarin_s_forward(0, nil, neg)
        with self.assertRaisesRegex(Exception, "Unknown style ID: -1"):
            core.yukarin_sa_forward(0, nil2, nil2, nil2, nil2, nil2, nil2, neg)
        with self.assertRaisesRegex(Exception, "Unknown style ID: -1"):
            core.decode_forward(0, 0, fnil2, fnil2, neg)
        core.finalize()

    def test_metas(self):
        with open(os.path.join(root_dir, "metas.json"), encoding="utf-8") as f:
            metas = json.dumps(json.load(f), sort_keys=True)
        core.initialize(root_dir, False)
        core_metas = json.dumps(json.loads(core.metas()), sort_keys=True)
        core.finalize()
        self.assertEqual(metas, core_metas)

    def test_supported_devices(self):
        devices = json.loads(core.supported_devices())
        for expected_device in ["cpu", "cuda"]:
            self.assertIn(expected_device, devices)
        self.assertTrue(devices["cpu"])

if __name__ == '__main__':
    unittest.main()
