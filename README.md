# Flow Aim - Opinionated SxT CLI setup tool

When trying to scaffold/setup your cryptographic verified analytics pipeline with SxT this tool is probably for you.

## CLI help 
```
Easily setup and load fake data in your SxT cryptographically verified analytics pipeline

Usage: flowaim [OPTIONS] <COMMAND>

Commands:
  setup    Setup schema and tables for test/production environments
  load     Load data into tables
  stats    Show stats
  destroy  Destroy schema
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --debug...  Turn debugging information on
  -h, --help      Print help
  -V, --version   Print version
```

## Setup

Ask your organisation name and sets up different tables based on how many environment you want to have. Options are dev/test/stag/prod.
```
.
├── ...
├── .flowaim                # Folder gets automatically added to your .gitignore file
│   ├── config.toml         # Your user_id, private key, public key and api url for SxT
│   ├── dev.toml            # Development table resource_name, biscuit and private/public key if you want to generate another biscuit
│   ├── test.toml           # Test table resource_name, biscuit and private/public key if you want to generate another biscuit
│   ├── stag.toml           # Staging table resource_name, biscuit and private/public key if you want to generate another biscuit
│   └── prod.toml           # Production table resource_name, biscuit and private/public key if you want to generate another biscuit
└── ...
```
## Load

Initial load of the dummy data in your table for testing. 

## Stats

Prints out a table with the last 7 days and the event types:
```
Events in the last 7 days:
+------------+------------+-------+
| date       | event_type | count |
+------------+------------+-------+
| 2023-06-04 | deck_start | 120   |
+------------+------------+-------+
| 2023-06-04 | deck_end   | 101   |
+------------+------------+-------+
| 2023-06-05 | deck_start | 80    |
+------------+------------+-------+
| 2023-06-05 | deck_end   | 78    |
+------------+------------+-------+
| 2023-06-06 | deck_start | 111   |
+------------+------------+-------+
| 2023-06-06 | deck_end   | 80    |
+------------+------------+-------+
| 2023-06-06 | ad_view    | 79    |
+------------+------------+-------+
| 2023-06-07 | deck_start | 25    |
+------------+------------+-------+
| 2023-06-07 | deck_end   | 20    |
+------------+------------+-------+
| 2023-06-07 | ad_view    | 18    |
+------------+------------+-------+
| 2023-06-07 | ad_click   | 7     |
+------------+------------+-------+
...
```
## Destroy

Destroy the schema after input. 
