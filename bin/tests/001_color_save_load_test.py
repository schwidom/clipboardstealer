#!/usr/bin/python3 -i

import tempfile
import os 


with tempfile.TemporaryDirectory() as temp_dir:
    print("Temporary directory:", temp_dir)
    os.system( f"cargo run -- --save-color-theme {temp_dir}/default.json")
    os.system( f"cargo run -- --save-color-theme {temp_dir}/dracula.json -c dracula")

    res = 0 == os.system( f"cmp {temp_dir}/default.json testdata/colorthemes/default.json")
    print( f"default ok : {res}")

    res = 0 == os.system( f"cmp {temp_dir}/dracula.json testdata/colorthemes/dracula.json")
    print( f"dracula ok : {res}")

    print(f"Temporary directory: {temp_dir}, press enter to clean up")
    # You can work with the directory here
    input()
    # Directory is automatically deleted after the block


