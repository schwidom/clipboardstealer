#!/bin/bash

stty --all >tmp/stty_start.txt 

cargo run 

stty --all >tmp/stty_stop.txt

