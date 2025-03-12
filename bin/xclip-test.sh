#!/bin/bash

echo primary | xclip -i -selection primary
echo clipboard | xclip -i -selection clipboard
echo secondary | xclip -i -selection secondary

