# Trace Integration Tests

Trace tests run WASM apps with scripted input and compare the resulting host API call trace.

## Run

```bash
./build_trace_tests.sh
```

Outputs go to `tests/trace/output/`.

## Golden Traces

Some specs use `expected_mode: "exact"` and compare against golden traces in
`tests/trace/expected/<app>/<spec>.json`.

To update after intentional changes:

```bash
./build_trace_tests.sh
cp tests/trace/output/<app>/<spec>.json tests/trace/expected/<app>/<spec>.json
```
