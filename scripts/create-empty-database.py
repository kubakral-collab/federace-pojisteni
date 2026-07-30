import argparse
import sqlite3
from pathlib import Path


SCHEMA = r'''
CREATE TABLE "Seznam" (
  "Identifikátor" INTEGER NOT NULL,
  "PojištěníOd" TEXT,
  "PojištěníDo" TEXT,
  "RočPojistné" INTEGER,
  "PojistnáČástka" INTEGER,
  "PojistNespotř" INTEGER,
  "Kategorie" TEXT,
  "Ztráta" INTEGER NOT NULL DEFAULT 0,
  "KódOC" TEXT,
  "EvČíslo" INTEGER,
  "Titul" TEXT,
  "Příjmení" TEXT,
  "Jméno" TEXT,
  "RodnéČíslo" TEXT,
  "Město" TEXT,
  "Adresa" TEXT,
  "PSČ" TEXT,
  "Stát" TEXT,
  "Poznámka" TEXT,
  "OdbPříslušnost" TEXT,
  "ZO" TEXT,
  "Ukončení" TEXT,
  "SkutÚhrada" INTEGER,
  "Doklad" INTEGER,
  "e-mail" TEXT,
  "Tisk" INTEGER NOT NULL DEFAULT 0,
  "DatumTisku" TEXT
);
CREATE TABLE "Editace" (
  "PojištěníOd" TEXT, "PojištěníDo" TEXT, "RočPojistné" INTEGER,
  "Kategorie" TEXT, "Ztráta" INTEGER NOT NULL DEFAULT 0,
  "PojistnáČástka" INTEGER, "KódOC" TEXT, "EvČíslo" INTEGER,
  "Titul" TEXT, "Příjmení" TEXT, "Jméno" TEXT, "RodnéČíslo" TEXT,
  "Město" TEXT, "Adresa" TEXT, "PSČ" TEXT, "Stát" TEXT,
  "Poznámka" TEXT, "OdbPříslušnost" TEXT, "ZO" TEXT,
  "Ukončení" TEXT, "SkutÚhrada" INTEGER, "E-mail" TEXT
);
CREATE TABLE "Kategorie" (
  "ID" INTEGER NOT NULL, "Kategorie" TEXT, "Ztráta" INTEGER,
  "Roč_Částka" INTEGER, "Pojistné" INTEGER, "Období" INTEGER
);
CREATE TABLE "PojistnaObdobi" (
  "Rok" INTEGER PRIMARY KEY,
  "Stav" TEXT NOT NULL CHECK ("Stav" IN ('AKTIVNI', 'UZAVRENO')),
  "Vytvoreno" TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO "PojistnaObdobi" ("Rok", "Stav") VALUES (2026, 'AKTIVNI');
'''


def main() -> None:
    parser = argparse.ArgumentParser(description="Create a data-free SQLite resource for CI builds.")
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    destination = args.destination.resolve()
    if destination.exists():
        raise SystemExit(f"Refusing to overwrite existing file: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    connection = sqlite3.connect(destination)
    try:
        connection.executescript(SCHEMA)
        connection.commit()
        integrity = connection.execute("PRAGMA quick_check").fetchone()[0]
        count = connection.execute('SELECT COUNT(*) FROM "Seznam"').fetchone()[0]
        if integrity != "ok" or count != 0:
            raise RuntimeError("Generated database failed validation")
    finally:
        connection.close()


if __name__ == "__main__":
    main()
