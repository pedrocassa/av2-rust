## Build
`cargo +nightly contract build`

## Build E2E
`cargo +nightly contract build --features e2e-tests`

## Test
`cargo +nightly test`

## Test E2E
`cargo +nightly test --features e2e-tests`

## Rodar o nó local
`substrate-contracts-node --log info,runtime::contracts=debug 2>&1`
