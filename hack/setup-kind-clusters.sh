#!/bin/bash

# Setup script to create 3 kind clusters for integration testing

set -e

# Check if kind is installed
if ! command -v kind &> /dev/null; then
    echo "kind is not installed. Please install kind first."
    echo "Visit: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
    exit 1
fi

# Check if clusters already exist
existing_clusters=$(kind get clusters 2>/dev/null || true)
if echo "$existing_clusters" | grep -q "kind-test1\|kind-test2\|kind-test3"; then
    echo "Some test clusters already exist. Please run teardown script first or delete manually."
    exit 1
fi

echo "Creating kind clusters for testing..."

# Create cluster 1
echo "Creating kind-test1..."
kind create cluster --name kind-test1 --wait 60s

# Create cluster 2
echo "Creating kind-test2..."
kind create cluster --name kind-test2 --wait 60s

# Create cluster 3
echo "Creating kind-test3..."
kind create cluster --name kind-test3 --wait 60s

echo "All clusters created successfully!"
echo "Available contexts:"
kubectl config get-contexts -o name | grep kind-test

echo ""
echo "To run integration tests:"
echo "cargo test --test integration"
echo ""
echo "To teardown:"
echo "./hack/teardown-kind-clusters.sh"