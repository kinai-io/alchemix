curl 




#!/bin/bash

# Define the URL where you want to send the POST request
url="http://localhost:8000/api/flux/adder/event"

# Define the JSON body
json_body='{"id":"0ad24096-58a3-4a7b-90f1-f0016f4ca7bd","kind":"AddAction","left":2,"right":3}'

# Use cURL to send a POST request with the JSON body
curl -X POST \
     -H "Content-Type: application/json" \
     -d "$json_body" \
     "$url"
