"""Read-only comparison of the authoritative Access member table and dd.sqlite.

The report intentionally emits internal identifiers and changed column names only.
It never prints personal field values.
"""

from __future__ import annotations

import argparse
import collections
import datetime as dt
import decimal
import hashlib
import json
import shutil
import sqlite3
import tempfile
from pathlib import Path
from typing import Any, Iterable

import pyodbc


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def normalize(value: Any) -> Any:
    if value is None:
        return None
    if isinstance(value, bool):
        return 1 if value else 0
    if isinstance(value, dt.datetime):
        return value.strftime("%Y-%m-%d %H:%M:%S")
    if isinstance(value, dt.date):
        return value.strftime("%Y-%m-%d 00:00:00")
    if isinstance(value, decimal.Decimal):
        return format(value.normalize(), "f")
    if isinstance(value, float):
        return format(decimal.Decimal(str(value)).normalize(), "f")
    if isinstance(value, bytes):
        return {"blobSha256": hashlib.sha256(value).hexdigest(), "size": len(value)}
    return value


def canonical(row: Iterable[Any]) -> tuple[Any, ...]:
    return tuple(normalize(value) for value in row)


def load_access(path: Path) -> tuple[list[str], list[tuple[Any, ...]]]:
    with tempfile.TemporaryDirectory(prefix="federace-access-audit-") as directory:
        copy = Path(directory) / "source.accdb"
        shutil.copy2(path, copy)
        connection = pyodbc.connect(
            "DRIVER={Microsoft Access Driver (*.mdb, *.accdb)};"
            f"DBQ={copy};READONLY=TRUE;",
            autocommit=True,
        )
        try:
            cursor = connection.cursor()
            cursor.execute("SELECT * FROM [Seznam]")
            columns = [item[0] for item in cursor.description]
            rows = [canonical(row) for row in cursor.fetchall()]
        finally:
            connection.close()
    return columns, rows


def load_sqlite(path: Path) -> tuple[list[str], list[tuple[Any, ...]]]:
    uri = f"file:{path.resolve().as_posix()}?mode=ro"
    connection = sqlite3.connect(uri, uri=True)
    try:
        cursor = connection.execute('SELECT * FROM "Seznam"')
        columns = [item[0] for item in cursor.description]
        rows = [canonical(row) for row in cursor.fetchall()]
    finally:
        connection.close()
    return columns, rows


def expand(counter: collections.Counter[tuple[Any, ...]]) -> list[tuple[Any, ...]]:
    return [row for row, count in counter.items() for _ in range(count)]


def compare(access: Path, sqlite: Path) -> dict[str, Any]:
    access_columns, access_rows = load_access(access)
    sqlite_columns, sqlite_rows = load_sqlite(sqlite)
    if access_columns != sqlite_columns[: len(access_columns)]:
        raise RuntimeError("Pořadí původních sloupců tabulky Seznam se neshoduje.")

    # SQLite may contain additive application columns such as Telefon. They do not
    # participate in comparison with the original Access schema.
    sqlite_rows = [row[: len(access_columns)] for row in sqlite_rows]
    access_counter = collections.Counter(access_rows)
    sqlite_counter = collections.Counter(sqlite_rows)
    exact_matches = sum((access_counter & sqlite_counter).values())
    remaining_access = expand(access_counter - sqlite_counter)
    remaining_sqlite = expand(sqlite_counter - access_counter)

    key_columns = ("Identifikátor", "EvČíslo")
    key_indexes = [access_columns.index(column) for column in key_columns]
    def business_key(row: tuple[Any, ...]) -> str:
        return "|".join(str(row[index]) for index in key_indexes)

    access_groups: dict[str, list[tuple[Any, ...]]] = collections.defaultdict(list)
    sqlite_groups: dict[str, list[tuple[Any, ...]]] = collections.defaultdict(list)
    for row in remaining_access:
        access_groups[business_key(row)].append(row)
    for row in remaining_sqlite:
        sqlite_groups[business_key(row)].append(row)

    changed: dict[str, list[str]] = {}
    field_counts: collections.Counter[str] = collections.Counter()
    ambiguous: dict[str, dict[str, int]] = {}
    only_access: list[str] = []
    only_sqlite: list[str] = []
    for identifier in sorted(set(access_groups) | set(sqlite_groups)):
        left_rows = access_groups.get(identifier, [])
        right_rows = sqlite_groups.get(identifier, [])
        if len(left_rows) == 1 and len(right_rows) == 1:
            left, right = left_rows[0], right_rows[0]
        elif not right_rows:
            only_access.extend(
                f"{identifier}|{row[access_columns.index('PojištěníOd')]}" for row in left_rows
            )
            continue
        elif not left_rows:
            only_sqlite.extend(
                f"{identifier}|{row[access_columns.index('PojištěníOd')]}" for row in right_rows
            )
            continue
        else:
            ambiguous[identifier] = {"access": len(left_rows), "sqlite": len(right_rows)}
            continue
        fields = [
            column
            for column, left_value, right_value in zip(access_columns, left, right)
            if left_value != right_value
        ]
        if fields:
            changed[identifier] = fields
            field_counts.update(fields)

    return {
        "access": {
            "path": str(access),
            "sha256": file_hash(access),
            "rows": len(access_rows),
        },
        "sqlite": {
            "path": str(sqlite),
            "sha256": file_hash(sqlite),
            "rows": len(sqlite_rows),
        },
        "columnsCompared": access_columns,
        "comparisonKey": list(key_columns),
        "additiveSqliteColumns": sqlite_columns[len(access_columns) :],
        "exactMatchingRows": exact_matches,
        "onlyInAccess": sorted(only_access),
        "onlyInSqlite": sorted(only_sqlite),
        "changedRows": changed,
        "changedFieldCounts": dict(sorted(field_counts.items())),
        "ambiguousRemainingGroups": ambiguous,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--access", required=True, type=Path)
    parser.add_argument("--sqlite", required=True, type=Path)
    args = parser.parse_args()
    result = compare(args.access, args.sqlite)
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
