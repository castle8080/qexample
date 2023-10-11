#!/bin/bash

exec cargo run -- --mode producer --credentials ../aad_credentials.json --queue testlog --namespace castle-rtestapp --count 1
