# Hack Scripts

This directory contains utility scripts for development and testing.

## Setup Kind Clusters

To set up 3 kind clusters for integration testing:

```bash
./hack/setup-kind-clusters.sh
```

This will create clusters named `kind-test1`, `kind-test2`, and `kind-test3`.

## Teardown Kind Clusters

To clean up the test clusters:

```bash
./hack/teardown-kind-clusters.sh
```

## Running Integration Tests

After setting up clusters, run the integration tests:

```bash
make test-integration
```

The tests will use the kind clusters to verify functionality.
