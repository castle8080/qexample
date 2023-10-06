#!/bin/bash

exec cargo run -- --mode consumer --credentials aad_credentials.json --queue testlog --namespace castle-rtestapp
