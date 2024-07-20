#!/bin/bash

seq 100 | xargs -iXX echo 'url = "http://127.0.0.1:8090/api/test/counter/"' | curl -X POST -I --parallel --parallel-immediate --parallel-max 20 --config -
