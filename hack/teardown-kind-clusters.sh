#!/bin/bash

# Teardown script to delete the test kind clusters

set -e

echo "Deleting test kind clusters..."

# Delete clusters if they exist
clusters=("mx-test1" "mx-test2" "mx-test3")
for cluster in "${clusters[@]}"; do
    if kind get clusters | grep -qxF "$cluster"; then
        echo "Deleting $cluster..."
        kind delete cluster --name "$cluster"
    else
        echo "$cluster does not exist, skipping..."
    fi
done

echo "Teardown complete!"