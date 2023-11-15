#!/bin/bash
name=${1?name}
(echo -ne '\x00'; convert "src/${name}.png" mono:-) > "src/${name}.icon"
