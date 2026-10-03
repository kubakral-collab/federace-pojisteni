import argparse
import hashlib
import json
import sqlite3
from pathlib import Path


def quoted(identifier: str) -> str:
    return '"' + identifier.replace('"', '""') + '"'


def encode(value) -> bytes:
    if value is None:
        payload = b""
        kind = b"N"
    elif isinstance(value, bytes):
        payload = value
        kind = b"B"
    else:
        payload = str(value).encode("utf-8")
        kind = type(value).__name__[0].upper().encode("ascii")
    return kind + str(len(payload)).encode("ascii") + b":" + payload + b";"


def table_digest(connection: sqlite3.Connection, table: str, columns: list[str]) -> tuple[int, str]:
    select = ",".join(quoted(column) for column in columns)
    order = ",".join(quoted(column) for column in columns)
    digest = hashlib.sha256()
    count = 0
    for row in connection.execute(f"SELECT {select} FROM {quoted(table)} ORDER BY {order}"):
        count += 1
        for value in row:
            digest.update(encode(value))
        digest.update(b"\n")
    return count, digest.hexdigest()


def scalar(connection: sqlite3.Connection, sql: str):
    try:
        return connection.execute(sql).fetchone()[0]
    except sqlite3.Error:
        return None


def snapshot(database: Path, baseline: dict | None) -> dict:
    connection = sqlite3.connect(f"file:{database.as_posix()}?mode=ro", uri=True)
    try:
        tables = [
            row[0]
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
            )
        ]
        table_data = {}
        baseline_tables = baseline.get("tables", {}) if baseline else {}
        selected_tables = list(baseline_tables) if baseline else tables
        for table in selected_tables:
            if table not in tables:
                table_data[table] = {"missing": True}
                continue
            actual_columns = [row[1] for row in connection.execute(f"PRAGMA table_info({quoted(table)})")]
            columns = baseline_tables.get(table, {}).get("columns", actual_columns)
            if any(column not in actual_columns for column in columns):
                table_data[table] = {"missingColumns": sorted(set(columns) - set(actual_columns))}
                continue
            count, digest = table_digest(connection, table, columns)
            table_data[table] = {"columns": columns, "rowCount": count, "sha256": digest}

        aggregates = {
            "members": scalar(connection, 'SELECT COUNT(*) FROM "Seznam"'),
            "uniqueMembers": scalar(connection, 'SELECT COUNT(DISTINCT "Identifikátor") FROM "Seznam"'),
            "organizations": scalar(connection, 'SELECT COUNT(DISTINCT NULLIF(TRIM("ZO"),\'\')) FROM "Seznam"'),
            "annualPremiumSum": scalar(connection, 'SELECT COALESCE(SUM(CAST("RočPojistné" AS INTEGER)),0) FROM "Seznam"'),
            "prescribedPremiumSum": scalar(connection, 'SELECT COALESCE(SUM(CAST("PojistnáČástka" AS INTEGER)),0) FROM "Seznam"'),
            "storedPaymentSum": scalar(connection, 'SELECT COALESCE(SUM(CAST("SkutÚhrada" AS INTEGER)),0) FROM "Seznam"'),
            "payments": scalar(connection, 'SELECT COUNT(*) FROM "PlatbyClenu"'),
            "paymentSum": scalar(connection, 'SELECT COALESCE(SUM("Castka"),0) FROM "PlatbyClenu"'),
            "claims": scalar(connection, 'SELECT COUNT(*) FROM "PojistneUdalosti"'),
            "legacyClaims": scalar(connection, 'SELECT COUNT(*) FROM "Poj_udalost"'),
            "receipts": scalar(connection, 'SELECT COUNT(*) FROM "DokladyOUhrade"'),
            "receiptSum": scalar(connection, 'SELECT COALESCE(SUM("Castka"),0) FROM "DokladyOUhrade"'),
            "invoices": scalar(connection, 'SELECT COUNT(*) FROM "VydaneFaktury"'),
            "invoiceSum": scalar(connection, 'SELECT COALESCE(SUM("Castka"),0) FROM "VydaneFaktury"'),
            "organizationPayments": scalar(connection, 'SELECT COUNT(*) FROM "OrganizacniPlatby"'),
            "organizationPaymentSum": scalar(connection, 'SELECT COALESCE(SUM("PrijataCastka"),0) FROM "OrganizacniPlatby"'),
        }
        return {
            "database": str(database.resolve()),
            "size": database.stat().st_size,
            "userVersion": scalar(connection, "PRAGMA user_version"),
            "integrityCheck": scalar(connection, "PRAGMA integrity_check"),
            "aggregates": aggregates,
            "tables": table_data,
        }
    finally:
        connection.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("database", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    baseline = json.loads(args.baseline.read_text(encoding="utf-8")) if args.baseline else None
    result = snapshot(args.database, baseline)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    if baseline:
        differences = []
        if baseline["aggregates"] != result["aggregates"]:
            differences.append("aggregates")
        for table, before in baseline["tables"].items():
            after = result["tables"].get(table)
            if not after or before.get("rowCount") != after.get("rowCount") or before.get("sha256") != after.get("sha256"):
                differences.append(f"table:{table}")
        print(json.dumps({"differences": differences, "userVersion": result["userVersion"], "integrityCheck": result["integrityCheck"]}, ensure_ascii=False))
    else:
        print(json.dumps({"aggregates": result["aggregates"], "userVersion": result["userVersion"], "integrityCheck": result["integrityCheck"], "tables": len(result["tables"])}, ensure_ascii=False))


if __name__ == "__main__":
    main()
