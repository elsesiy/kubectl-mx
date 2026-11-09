#!/bin/bash

# Setup script to create 3 kind clusters for integration testing.
# Idempotent: clusters that already exist are kept, so it is safe in CI and locally.

set -euo pipefail

clusters=("mx-test1" "mx-test2" "mx-test3")

# Check if kind is installed
if ! command -v kind &> /dev/null; then
    echo "kind is not installed. Please install kind first."
    echo "Visit: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
    exit 1
fi

existing_clusters=$(kind get clusters 2>/dev/null || true)

for cluster in "${clusters[@]}"; do
    if grep -qxF "$cluster" <<< "$existing_clusters"; then
        echo "$cluster already exists, skipping..."
    else
        echo "Creating $cluster..."
        kind create cluster --name "$cluster" --wait 120s
    fi
done

echo "All clusters are ready!"
echo "Available contexts:"
kubectl config get-contexts -o name | grep kind-mx-test

echo ""
echo "To run integration tests:"
echo "cargo test --test integration"
echo ""
echo "To teardown:"
echo "./hack/teardown-kind-clusters.sh"
