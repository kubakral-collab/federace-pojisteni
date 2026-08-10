use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationMember {
    pub row_id: i64,
    pub identifier: String,
    pub name: String,
    pub registration_number: String,
    pub expected: i64,
    pub paid: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationOption {
    pub name: String,
    pub member_count: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationInput {
    pub row_id: i64,
    pub amount: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPaymentInput {
    pub organization: String,
    pub received_on: String,
    pub insurance_year: i32,
    pub received_amount: i64,
    pub note: Option<String>,
    pub allocations: Vec<AllocationInput>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPayment {
    pub id: i64,
    pub organization: String,
    pub received_on: String,
    pub insurance_year: i32,
    pub received_amount: i64,
    pub expected_amount: i64,
    pub unassigned_overpayment: i64,
    pub note: Option<String>,
    pub member_count: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Allocation {
    pub member_row_id: i64,
    pub member_name: String,
    pub registration_number: String,
    pub amount: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPaymentDetail {
    pub payment: OrganizationPayment,
    pub allocations: Vec<Allocation>,
}

pub fn ensure_schema(c: &Connection) -> rusqlite::Result<()> {
    c.execute_batch(r#"CREATE TABLE IF NOT EXISTS "OrganizacniPlatby"(
 "Id" INTEGER PRIMARY KEY AUTOINCREMENT,"Organizace" TEXT NOT NULL,"DatumPrijeti" TEXT NOT NULL,
 "PojistnyRok" INTEGER NOT NULL,"PrijataCastka" INTEGER NOT NULL CHECK("PrijataCastka">0),
 "OcekavanaCastka" INTEGER NOT NULL,"NeprirazenyPreplatek" INTEGER NOT NULL DEFAULT 0,"Poznamka" TEXT,
 "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
 CREATE TABLE IF NOT EXISTS "RozpisOrganizacniPlatby"("Id" INTEGER PRIMARY KEY AUTOINCREMENT,
 "OrganizacniPlatbaId" INTEGER NOT NULL,"PojistnyZaznamRowId" INTEGER NOT NULL,"IdentifikatorClena" TEXT NOT NULL,
 "Castka" INTEGER NOT NULL CHECK("Castka">=0), UNIQUE("OrganizacniPlatbaId","PojistnyZaznamRowId"));
 CREATE INDEX IF NOT EXISTS "IX_RozpisOrganizacniPlatby_Hlavni" ON "RozpisOrganizacniPlatby"("OrganizacniPlatbaId");"#)
}

pub fn organizations(c: &Connection, year: i32) -> rusqlite::Result<Vec<OrganizationOption>> {
    let mut s=c.prepare(r#"SELECT TRIM("ZO"),COUNT(*) FROM "Seznam" WHERE CAST(substr("PojištěníOd",1,4) AS INTEGER)=?1 AND NULLIF(TRIM("Ukončení"),'') IS NULL AND NULLIF(TRIM("ZO"),'') IS NOT NULL GROUP BY TRIM("ZO") ORDER BY TRIM("ZO")"#)?;
    let result = s
        .query_map([year], |r| {
            Ok(OrganizationOption {
                name: r.get(0)?,
                member_count: r.get(1)?,
            })
        })?
        .collect();
    result
}
pub fn members(c: &Connection, org: &str, year: i32) -> rusqlite::Result<Vec<OrganizationMember>> {
    let mut s=c.prepare(r#"SELECT rowid,COALESCE(CAST("Identifikátor" AS TEXT),''),TRIM(COALESCE("Titul",'')||' '||COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')),COALESCE(CAST("EvČíslo" AS TEXT),''),COALESCE("RočPojistné",0),COALESCE("SkutÚhrada",0) FROM "Seznam" WHERE CAST(substr("PojištěníOd",1,4) AS INTEGER)=?1 AND NULLIF(TRIM("Ukončení"),'') IS NULL AND TRIM("ZO")=TRIM(?2) ORDER BY CAST("EvČíslo" AS INTEGER),"Příjmení""#)?;
    let result = s
        .query_map(params![year, org], |r| {
            Ok(OrganizationMember {
                row_id: r.get(0)?,
                identifier: r.get(1)?,
                name: r.get(2)?,
                registration_number: r.get(3)?,
                expected: r.get(4)?,
                paid: r.get(5)?,
            })
        })?
        .collect();
    result
}
pub fn save(path: &Path, input: OrganizationPaymentInput) -> Result<i64, String> {
    chrono::NaiveDate::parse_from_str(input.received_on.trim(), "%Y-%m-%d")
        .map_err(|_| "Zkontrolujte datum platby.".to_string())?;
    if input.organization.trim().is_empty()
        || input.received_amount <= 0
        || input.allocations.is_empty()
    {
        return Err("Doplňte organizaci, částku a alespoň jednoho člena.".into());
    }
    let mut c = Connection::open(path)
        .map_err(|_| "Organizační platbu se nepodařilo uložit.".to_string())?;
    ensure_schema(&c)
        .map_err(|_| "Databázi organizačních plateb se nepodařilo připravit.".to_string())?;
    crate::member_payments::ensure_schema(&c)
        .map_err(|_| "Databázi plateb se nepodařilo připravit.".to_string())?;
    let valid = members(&c, &input.organization, input.insurance_year)
        .map_err(|_| "Členy organizace se nepodařilo ověřit.".to_string())?;
    let mut expected = 0;
    let mut allocated = 0;
    for a in &input.allocations {
        let m = valid
            .iter()
            .find(|m| m.row_id == a.row_id)
            .ok_or("Rozpis obsahuje člena mimo vybranou organizaci.")?;
        if a.amount < 0 || a.amount > m.expected {
            return Err(format!(
                "Částka u člena {} musí být mezi 0 a jeho pojistným.",
                m.name
            ));
        }
        expected += m.expected;
        allocated += a.amount;
    }
    let over = (input.received_amount - expected).max(0);
    let required = input.received_amount - over;
    if allocated != required {
        return Err(format!(
            "Součet rozpisu musí být {} Kč; nyní je {} Kč.",
            required, allocated
        ));
    }
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Organizační platbu se nepodařilo uložit.".to_string())?;
    tx.execute(r#"INSERT INTO "OrganizacniPlatby"("Organizace","DatumPrijeti","PojistnyRok","PrijataCastka","OcekavanaCastka","NeprirazenyPreplatek","Poznamka") VALUES(?1,?2,?3,?4,?5,?6,?7)"#,params![input.organization.trim(),input.received_on.trim(),input.insurance_year,input.received_amount,expected,over,input.note]).map_err(|_|"Hlavní organizační platbu se nepodařilo uložit.".to_string())?;
    let id = tx.last_insert_rowid();
    for a in input.allocations.iter().filter(|a| a.amount > 0) {
        let m = valid.iter().find(|m| m.row_id == a.row_id).unwrap();
        tx.execute(r#"INSERT INTO "RozpisOrganizacniPlatby"("OrganizacniPlatbaId","PojistnyZaznamRowId","IdentifikatorClena","Castka")VALUES(?1,?2,?3,?4)"#,params![id,a.row_id,m.identifier,a.amount]).map_err(|_|"Rozpis se nepodařilo uložit.".to_string())?;
        tx.execute(r#"INSERT INTO "PlatbyClenu"("IdentifikatorClena","PojistnyZaznamRowId","PojistnyRok","DatumPrijeti","Castka","ZpusobUhrady","VariabilniSymbol","Poznamka","OrganizacniPlatbaId")VALUES(?1,?2,?3,?4,?5,'Organizace','',?6,?7)"#,params![m.identifier,a.row_id,input.insurance_year,input.received_on.trim(),a.amount,input.note,id]).map_err(|_|"Platbu člena se nepodařilo uložit.".to_string())?;
        tx.execute(r#"UPDATE "Seznam" SET "SkutÚhrada"=COALESCE((SELECT SUM("Castka") FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId"=?1),0) WHERE rowid=?1"#,[a.row_id]).map_err(|_|"Úhradu člena se nepodařilo přepočítat.".to_string())?;
    }
    tx.commit()
        .map_err(|_| "Organizační platbu se nepodařilo uložit.".to_string())?;
    Ok(id)
}
pub fn list(c: &Connection) -> rusqlite::Result<Vec<OrganizationPayment>> {
    let mut s=c.prepare(r#"SELECT p."Id",p."Organizace",p."DatumPrijeti",p."PojistnyRok",p."PrijataCastka",p."OcekavanaCastka",p."NeprirazenyPreplatek",p."Poznamka",COUNT(r."Id") FROM "OrganizacniPlatby" p LEFT JOIN "RozpisOrganizacniPlatby" r ON r."OrganizacniPlatbaId"=p."Id" GROUP BY p."Id" ORDER BY p."DatumPrijeti" DESC,p."Id" DESC"#)?;
    let result = s
        .query_map([], |r| {
            Ok(OrganizationPayment {
                id: r.get(0)?,
                organization: r.get(1)?,
                received_on: r.get(2)?,
                insurance_year: r.get(3)?,
                received_amount: r.get(4)?,
                expected_amount: r.get(5)?,
                unassigned_overpayment: r.get(6)?,
                note: r.get(7)?,
                member_count: r.get(8)?,
            })
        })?
        .collect();
    result
}
pub fn detail(c: &Connection, id: i64) -> rusqlite::Result<OrganizationPaymentDetail> {
    let payment = list(c)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let mut s=c.prepare(r#"SELECT r."PojistnyZaznamRowId",TRIM(COALESCE(m."Titul",'')||' '||COALESCE(m."Příjmení",'')||' '||COALESCE(m."Jméno",'')),COALESCE(CAST(m."EvČíslo" AS TEXT),''),r."Castka" FROM "RozpisOrganizacniPlatby" r JOIN "Seznam" m ON m.rowid=r."PojistnyZaznamRowId" WHERE r."OrganizacniPlatbaId"=?1 ORDER BY CAST(m."EvČíslo" AS INTEGER)"#)?;
    let allocations = s
        .query_map([id], |r| {
            Ok(Allocation {
                member_row_id: r.get(0)?,
                member_name: r.get(1)?,
                registration_number: r.get(2)?,
                amount: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(OrganizationPaymentDetail {
        payment,
        allocations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::functions::FunctionFlags;

    fn database() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("organization-payments.sqlite");
        let c = Connection::open(&path).unwrap();
        c.create_scalar_function(
            "pojisteni_rok",
            1,
            FunctionFlags::SQLITE_DETERMINISTIC,
            |ctx| {
                let value: String = ctx.get(0)?;
                Ok(value.get(0..4).and_then(|v| v.parse::<i32>().ok()))
            },
        )
        .unwrap();
        c.execute_batch(r#"CREATE TABLE "Seznam"("Identifikátor" TEXT,"Titul" TEXT,"Příjmení" TEXT,"Jméno" TEXT,"EvČíslo" INTEGER,"RočPojistné" INTEGER,"SkutÚhrada" INTEGER,"ZO" TEXT,"PojištěníOd" TEXT,"Ukončení" TEXT);
        INSERT INTO "Seznam" VALUES('A','','Novák','Jan',1,2400,0,'ZO Test','2027-01-01',NULL);
        INSERT INTO "Seznam" VALUES('B','','Malá','Eva',2,1800,0,'ZO Test','2027-01-01',NULL);"#).unwrap();
        drop(c);
        (dir, path)
    }

    #[test]
    fn exact_payment_creates_parent_allocations_and_member_payments() {
        let (_dir, path) = database();
        let id = save(
            &path,
            OrganizationPaymentInput {
                organization: "ZO Test".into(),
                received_on: "2027-03-15".into(),
                insurance_year: 2027,
                received_amount: 4200,
                note: None,
                allocations: vec![
                    AllocationInput {
                        row_id: 1,
                        amount: 2400,
                    },
                    AllocationInput {
                        row_id: 2,
                        amount: 1800,
                    },
                ],
            },
        )
        .unwrap();
        let c = Connection::open(&path).unwrap();
        assert_eq!(detail(&c, id).unwrap().allocations.len(), 2);
        assert_eq!(
            c.query_row(r#"SELECT SUM("SkutÚhrada") FROM "Seznam""#, [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            4200
        );
    }

    #[test]
    fn underpayment_requires_exact_manual_allocation() {
        let (_dir, path) = database();
        let result = save(
            &path,
            OrganizationPaymentInput {
                organization: "ZO Test".into(),
                received_on: "2027-03-15".into(),
                insurance_year: 2027,
                received_amount: 3000,
                note: None,
                allocations: vec![
                    AllocationInput {
                        row_id: 1,
                        amount: 2000,
                    },
                    AllocationInput {
                        row_id: 2,
                        amount: 500,
                    },
                ],
            },
        );
        assert!(result.unwrap_err().contains("Součet rozpisu"));
    }

    #[test]
    fn overpayment_remains_unassigned() {
        let (_dir, path) = database();
        let id = save(
            &path,
            OrganizationPaymentInput {
                organization: "ZO Test".into(),
                received_on: "2027-03-15".into(),
                insurance_year: 2027,
                received_amount: 5000,
                note: None,
                allocations: vec![
                    AllocationInput {
                        row_id: 1,
                        amount: 2400,
                    },
                    AllocationInput {
                        row_id: 2,
                        amount: 1800,
                    },
                ],
            },
        )
        .unwrap();
        let c = Connection::open(&path).unwrap();
        assert_eq!(detail(&c, id).unwrap().payment.unassigned_overpayment, 800);
    }
}
