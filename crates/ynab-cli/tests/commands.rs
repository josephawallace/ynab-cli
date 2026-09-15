use std::str::FromStr;

use clap::Parser;
use ynab_cli::{Cli, commands::CliAmount};
use ynab_sdk::Milliunits;

const PLAN: &str = "00000000-0000-0000-0000-000000000001";
const ID: &str = "00000000-0000-0000-0000-000000000002";
const TRANSACTION_ID: &str = "00000000-0000-0000-0000-000000000002_2026-09-13";

#[test]
fn every_documented_command_parses() {
    let commands: &[&[&str]] = &[
        &["ynab-cli", "user", "show"],
        &["ynab-cli", "plan", "list", "--include-accounts"],
        &["ynab-cli", "plan", "show", "--last-knowledge", "4"],
        &["ynab-cli", "plan", "settings", "--plan", "default"],
        &["ynab-cli", "account", "list", "--last-knowledge", "4"],
        &["ynab-cli", "account", "show", ID],
        &[
            "ynab-cli",
            "account",
            "create",
            "--plan",
            PLAN,
            "--name",
            "Checking",
            "--type",
            "checking",
            "--balance",
            "10.250",
        ],
        &["ynab-cli", "category", "list"],
        &["ynab-cli", "category", "show", ID],
        &[
            "ynab-cli",
            "category",
            "create",
            "--plan",
            PLAN,
            "--name",
            "Food",
            "--category-group",
            ID,
        ],
        &[
            "ynab-cli",
            "category",
            "update",
            ID,
            "--plan",
            PLAN,
            "--name",
            "Groceries",
        ],
        &["ynab-cli", "category", "month", "show", "2026-09-01", ID],
        &[
            "ynab-cli",
            "category",
            "month",
            "assign",
            "2026-09-01",
            ID,
            "--plan",
            PLAN,
            "--amount",
            "1.000",
        ],
        &[
            "ynab-cli",
            "category-group",
            "create",
            "--plan",
            PLAN,
            "--name",
            "Needs",
        ],
        &[
            "ynab-cli",
            "category-group",
            "update",
            ID,
            "--plan",
            PLAN,
            "--name",
            "Wants",
        ],
        &["ynab-cli", "payee", "list"],
        &["ynab-cli", "payee", "show", ID],
        &[
            "ynab-cli", "payee", "create", "--plan", PLAN, "--name", "Acme",
        ],
        &[
            "ynab-cli", "payee", "update", ID, "--plan", PLAN, "--name", "Acme 2",
        ],
        &["ynab-cli", "payee", "location", "list", "--payee", ID],
        &["ynab-cli", "payee", "location", "show", ID],
        &["ynab-cli", "month", "list"],
        &["ynab-cli", "month", "show", "2026-09-01"],
        &["ynab-cli", "money-movement", "list"],
        &[
            "ynab-cli",
            "money-movement",
            "list",
            "--month",
            "2026-09-01",
        ],
        &["ynab-cli", "money-movement-group", "list"],
        &[
            "ynab-cli",
            "money-movement-group",
            "list",
            "--month",
            "2026-09-01",
        ],
        &[
            "ynab-cli",
            "transaction",
            "list",
            "--category",
            ID,
            "--since",
            "2026-01-01",
            "--until",
            "2026-01-31",
            "--type",
            "uncategorized",
        ],
        &["ynab-cli", "transaction", "show", TRANSACTION_ID],
        &[
            "ynab-cli",
            "transaction",
            "create",
            "--plan",
            PLAN,
            "--account",
            ID,
            "--date",
            "2026-09-13",
            "--amount",
            "-1.250",
        ],
        &[
            "ynab-cli",
            "transaction",
            "create",
            "--plan",
            PLAN,
            "--input",
            "-",
        ],
        &[
            "ynab-cli",
            "transaction",
            "update",
            TRANSACTION_ID,
            "--plan",
            PLAN,
            "--memo",
            "updated",
        ],
        &[
            "ynab-cli",
            "transaction",
            "batch-update",
            "--plan",
            PLAN,
            "--input",
            "-",
            "--yes",
        ],
        &["ynab-cli", "transaction", "import", "--plan", PLAN, "--yes"],
        &[
            "ynab-cli",
            "transaction",
            "delete",
            TRANSACTION_ID,
            "--plan",
            PLAN,
            "--yes",
        ],
        &["ynab-cli", "scheduled-transaction", "list"],
        &["ynab-cli", "scheduled-transaction", "show", ID],
        &[
            "ynab-cli",
            "scheduled-transaction",
            "create",
            "--plan",
            PLAN,
            "--account",
            ID,
            "--date",
            "2026-09-13",
        ],
        &[
            "ynab-cli",
            "scheduled-transaction",
            "update",
            ID,
            "--plan",
            PLAN,
            "--account",
            ID,
            "--date",
            "2026-09-13",
        ],
        &[
            "ynab-cli",
            "scheduled-transaction",
            "delete",
            ID,
            "--plan",
            PLAN,
            "--yes",
        ],
        &["ynab-cli", "auth", "store"],
        &["ynab-cli", "auth", "clear"],
        &["ynab-cli", "auth", "status"],
    ];
    for command in commands {
        assert!(
            Cli::try_parse_from(command.iter().copied()).is_ok(),
            "failed to parse {command:?}"
        );
    }
}

#[test]
fn write_requires_explicit_plan() {
    assert!(Cli::try_parse_from(["ynab-cli", "payee", "create", "--name", "Acme"]).is_err());
}

#[test]
fn transaction_scopes_conflict_before_execution() {
    assert!(
        Cli::try_parse_from([
            "ynab-cli",
            "transaction",
            "list",
            "--account",
            ID,
            "--category",
            ID
        ])
        .is_err()
    );
}

#[test]
fn destructive_commands_require_yes() {
    assert!(
        Cli::try_parse_from(["ynab-cli", "transaction", "delete", ID, "--plan", PLAN]).is_err()
    );
    assert!(Cli::try_parse_from(["ynab-cli", "transaction", "import", "--plan", PLAN]).is_err());
}

#[test]
fn amount_conversion_is_exact() {
    assert_eq!(
        CliAmount::from_str("-0.001").expect("amount").0,
        Milliunits(-1)
    );
    assert_eq!(
        CliAmount::from_str("12.345").expect("amount").0,
        Milliunits(12_345)
    );
    assert!(CliAmount::from_str("12.3456").is_err());
}
