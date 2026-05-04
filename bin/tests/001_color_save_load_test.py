#!/usr/bin/python3 -i

import tempfile

with tempfile.TemporaryDirectory() as temp_dir:
    print("Temporary directory:", temp_dir)
    # You can work with the directory here
# Directory is automatically deleted after the block


