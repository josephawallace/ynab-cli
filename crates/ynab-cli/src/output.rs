use std::io::Write;

use comfy_table::{Table, presets::UTF8_FULL};
use serde_json::Value;

use crate::{commands::Output, error::CliError};

pub fn render(value: &Value, output: Output, writer: &mut dyn Write) -> Result<(), CliError> {
    match output {
        Output::Json => {
            serde_json::to_writer(&mut *writer, value)
                .map_err(|error| CliError::Local(error.to_string()))?;
            writeln!(writer)?;
        }
        Output::Table => render_table(value, writer)?,
    }
    Ok(())
}

const COLUMNS: &[&str] = &[
    "id",
    "name",
    "date",
    "month",
    "type",
    "account_name",
    "payee_name",
    "category_name",
    "amount",
    "balance",
    "budgeted",
    "activity",
    "cleared",
    "approved",
    "on_budget",
    "closed",
    "deleted",
    "frequency",
    "server_knowledge",
    "source",
];

fn render_table(value: &Value, writer: &mut dyn Write) -> Result<(), CliError> {
    let rows = table_rows(value);
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let keys: Vec<_> = COLUMNS
        .iter()
        .filter(|key| {
            rows.iter()
                .filter_map(|row| row.as_object())
                .any(|object| object.contains_key(**key))
        })
        .copied()
        .collect();
    if keys.is_empty() {
        table.set_header(vec!["value"]);
        for row in rows {
            table.add_row(vec![scalar_text(&row)]);
        }
    } else {
        table.set_header(keys.iter());
        for row in rows {
            let object = row.as_object();
            table.add_row(keys.iter().map(|key| {
                object
                    .and_then(|value| value.get(*key))
                    .map_or_else(String::new, scalar_text)
            }));
        }
    }
    writeln!(writer, "{table}")?;
    Ok(())
}

fn table_rows(value: &Value) -> Vec<Value> {
    const COLLECTIONS: &[&str] = &[
        "plans",
        "accounts",
        "category_groups",
        "categories",
        "payees",
        "payee_locations",
        "months",
        "money_movements",
        "money_movement_groups",
        "transactions",
        "scheduled_transactions",
    ];
    match value {
        Value::Array(rows) => rows.clone(),
        Value::Object(object) => COLLECTIONS
            .iter()
            .find_map(|key| object.get(*key).and_then(Value::as_array))
            .cloned()
            .unwrap_or_else(|| vec![value.clone()]),
        value => vec![value.clone()],
    }
}

fn scalar_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        value => serde_json::to_string(value).expect("JSON values always serialize"),
    }
}
