"""Read-only aggregate audit of Access and SQLite operational workflows.

The output contains counts and monetary aggregates only. It never emits names,
addresses, birth numbers, e-mails, or other personal field values.
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
from typing import Any

import pyodbc


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest().upper()


def normalize(value: Any) -> Any:
    if value is None:
        return None
    if isinstance(value, bool):
        return int(value)
    if isinstance(value, (dt.datetime, dt.date)):
        return value.strftime("%Y-%m-%d %H:%M:%S")
    if isinstance(value, (decimal.Decimal, float)):
        return str(decimal.Decimal(str(value)).normalize())
    return value


def load_access(path: Path) -> tuple[list[str], list[tuple[Any, ...]]]:
    with tempfile.TemporaryDirectory(prefix="federace-workflow-audit-") as directory:
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
            columns = [column[0] for column in cursor.description]
            rows = [tuple(normalize(value) for value in row) for row in cursor.fetchall()]
        finally:
            connection.close()
    return columns, rows


def load_sqlite(path: Path) -> tuple[list[str], list[tuple[Any, ...]], dict[str, int]]:
    connection = sqlite3.connect(f"file:{path.resolve().as_posix()}?mode=ro", uri=True)
    try:
        cursor = connection.execute('SELECT * FROM "Seznam"')
        columns = [column[0] for column in cursor.description]
        rows = [tuple(normalize(value) for value in row) for row in cursor.fetchall()]
        optional = {}
        for table in ("PlatbyClenu", "PlatebniPrikazy", "sazby_pojistneho"):
            exists = connection.execute(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
                (table,),
            ).fetchone()[0]
            optional[table] = connection.execute(
                f'SELECT COUNT(*) FROM "{table}"'
            ).fetchone()[0] if exists else -1
    finally:
        connection.close()
    return columns, rows, optional


def year(value: Any) -> int | None:
    text = str(value or "").strip()
    return int(text[:4]) if len(text) >= 4 and text[:4].isdigit() else None


def number(value: Any) -> decimal.Decimal:
    if value in (None, ""):
        return decimal.Decimal(0)
    return decimal.Decimal(str(value))


def summarize(columns: list[str], rows: list[tuple[Any, ...]]) -> dict[str, Any]:
    index = {name: columns.index(name) for name in (
        "PojištěníOd", "RočPojistné", "PojistnáČástka", "Kategorie",
        "Ztráta", "SkutÚhrada", "Ukončení",
    )}
    yearly: dict[int, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    tariffs: collections.Counter[str] = collections.Counter()
    for row in rows:
        insurance_year = year(row[index["PojištěníOd"]])
        if insurance_year is None:
            continue
        premium = number(row[index["PojistnáČástka"]])
        paid = number(row[index["SkutÚhrada"]])
        data = yearly[insurance_year]
        data["members"] += 1
        data["prescribed"] += premium
        data["paid"] += paid
        data["paidPositive"] += int(paid > 0)
        data["fullyPaid"] += int(paid == premium)
        data["underpaid"] += int(paid < premium)
        data["overpaid"] += int(paid > premium)
        data["terminated"] += int(bool(str(row[index["Ukončení"]] or "").strip()))
        tariff_key = "|".join(map(str, (
            insurance_year,
            row[index["RočPojistné"]],
            row[index["Kategorie"]],
            int(number(row[index["Ztráta"]]) != 0),
            row[index["PojistnáČástka"]],
        )))
        tariffs[tariff_key] += 1
    return {
        "yearly": {str(key): dict(value) for key, value in sorted(yearly.items())},
        "observedTariffTuples": dict(sorted(tariffs.items())),
    }


def changed_keys(left: dict[str, Any], right: dict[str, Any]) -> list[str]:
    return sorted(key for key in set(left) | set(right) if left.get(key) != right.get(key))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--access", required=True, type=Path)
    parser.add_argument("--sqlite", required=True, type=Path)
    args = parser.parse_args()
    access_columns, access_rows = load_access(args.access)
    sqlite_columns, sqlite_rows, optional = load_sqlite(args.sqlite)
    if access_columns != sqlite_columns[: len(access_columns)]:
        raise RuntimeError("Pořadí původních sloupců Seznam se neshoduje.")
    sqlite_rows = [row[: len(access_columns)] for row in sqlite_rows]
    access_summary = summarize(access_columns, access_rows)
    sqlite_summary = summarize(access_columns, sqlite_rows)
    changed_tariffs = changed_keys(
        access_summary["observedTariffTuples"],
        sqlite_summary["observedTariffTuples"],
    )
    result = {
        "access": {"sha256": file_hash(args.access), "rows": len(access_rows)},
        "sqlite": {"sha256": file_hash(args.sqlite), "rows": len(sqlite_rows)},
        "changedAggregateYears": changed_keys(
            access_summary["yearly"], sqlite_summary["yearly"]
        ),
        "changedObservedTariffTuples": {
            key: {
                "accessCount": access_summary["observedTariffTuples"].get(key, 0),
                "sqliteCount": sqlite_summary["observedTariffTuples"].get(key, 0),
            }
            for key in changed_tariffs
        },
        "accessYearly": access_summary["yearly"],
        "sqliteYearly": sqlite_summary["yearly"],
        "accessObservedTariffTupleCount": len(access_summary["observedTariffTuples"]),
        "sqliteObservedTariffTupleCount": len(sqlite_summary["observedTariffTuples"]),
        "sqliteOperationalTables": optional,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2, default=str))


if __name__ == "__main__":
    main()
