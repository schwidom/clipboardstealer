#!/bin/bash

grep -n '#\[test\]\|#\[cfg(test)\]\|\<mod\>\|\<fn test_' src/termionscreen.rs src/clipboards.rs

