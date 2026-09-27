# estimate-export

Reads a neutral `tpt-c-model` project JSON plus a CSV cost database, prices the
model with `tpt-c-estimating`'s `EstimateBuilder`, and writes the resulting
priced estimate to an `.xlsx` workbook. On success it prints the line-item
count and the subtotal/total for the estimate (currency defaults to USD).

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p estimate-export -- <model.json> <rates.csv> <out.xlsx>
# for example, against the golden fixtures:
cargo run -p estimate-export -- test-data/golden/sample-model.json \
    test-data/golden/rates.csv estimate.xlsx
```

All three arguments are required; missing ones print a usage message on stderr
and exit with a failure code.

Output (numbers depend on the model and rates):

```text
Wrote estimate.xlsx: <N> line items, subtotal <X.XX> USD, total <Y.YY> USD
```

## What it exercises

- `tpt_c_model::Project` — deserialized from the neutral model JSON.
- `tpt_c_csv::read_cost_database` — loads the cost database (code, title,
  kind, unit, rate, currency) from CSV.
- `tpt_c_cost::CostDatabase` and `tpt_c_estimating::EstimateBuilder` — map
  measured quantities to priced line items and compute subtotal/total.
- `tpt_c_xlsx::write_estimate` — exports the finished estimate as an Excel
  workbook.
- `tpt_c_quantities` / `tpt_c_core` — the quantity and money primitives that
  flow through the estimate.
- Error handling for unreadable files, invalid JSON/CSV, estimate failures,
  and workbook write errors, each reported on stderr with a failure exit code.

## License

Dual-licensed `MIT OR Apache-2.0`.
