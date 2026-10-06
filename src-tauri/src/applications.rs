use chrono::{Datelike, NaiveDate};
use printpdf::{Color, Line, Mm, PdfDocument, Point, Rgb};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationInput {
    pub first_name: String,
    pub last_name: String,
    pub personal_id: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub email: Option<String>,
    pub category: String,
    pub loss: bool,
    pub annual_amount: i64,
    pub affiliation: String,
    pub organization: String,
    pub code: String,
    pub registration_number: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationCodeOption {
    pub organization: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationOptions {
    pub organizations: Vec<OrganizationCodeOption>,
    pub annual_amounts: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationResult {
    pub application_id: i64,
    pub identifier: i64,
    pub registration_number: i64,
    pub application_date: String,
    pub insurance_from: String,
    pub premium: i64,
    pub pdf_path: String,
}

#[derive(Debug, Clone)]
pub struct PdfData {
    pub federation_name: String,
    pub account: String,
    pub contract_number: String,
    pub registration_number: String,
    pub first_name: String,
    pub last_name: String,
    pub personal_id: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub email: String,
    pub category: String,
    pub loss: bool,
    pub annual_amount: i64,
    pub premium: i64,
    pub application_date: String,
    pub blank: bool,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "Prihlasky" (
             "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
             "PojistnyZaznamRowId" INTEGER NOT NULL,
             "IdentifikatorClena" INTEGER NOT NULL,
             "DatumPrihlasky" TEXT NOT NULL,
             "PojisteniOd" TEXT NOT NULL,
             "Kategorie" TEXT NOT NULL,
             "Ztrata" INTEGER NOT NULL,
             "RocniLimit" INTEGER NOT NULL,
             "Pojistne" INTEGER NOT NULL,
             "Pdf" BLOB NOT NULL,
             "Sha256" TEXT NOT NULL,
             "CreatedAt" TEXT NOT NULL DEFAULT (datetime('now'))
           );
           CREATE INDEX IF NOT EXISTS "idx_prihlasky_clen" ON "Prihlasky"("IdentifikatorClena");
           CREATE TRIGGER IF NOT EXISTS "trg_prihlasky_immutable_update"
             BEFORE UPDATE ON "Prihlasky" BEGIN SELECT RAISE(ABORT, 'Přihláška je neměnný dokument.'); END;
           CREATE TRIGGER IF NOT EXISTS "trg_prihlasky_immutable_delete"
             BEFORE DELETE ON "Prihlasky" BEGIN SELECT RAISE(ABORT, 'Přihláška je neměnný dokument.'); END;"#,
    )
}

pub fn organizations(connection: &Connection, affiliation: &str) -> rusqlite::Result<Vec<OrganizationCodeOption>> {
    let mut statement = connection.prepare(
        r#"SELECT "ZO",
                  CASE WHEN COUNT(DISTINCT NULLIF(TRIM(CAST("KódOC" AS TEXT)), '')) = 1
                       THEN MIN(NULLIF(TRIM(CAST("KódOC" AS TEXT)), '')) ELSE NULL END
           FROM "Seznam"
           WHERE "OdbPříslušnost"=?1 AND NULLIF(TRIM("ZO"),'') IS NOT NULL
           GROUP BY "ZO" ORDER BY "ZO""#,
    )?;
    let result = statement
        .query_map([affiliation], |row| Ok(OrganizationCodeOption { organization: row.get(0)?, code: row.get(1)? }))?
        .collect();
    result
}

pub fn validate(input: &ApplicationInput) -> Result<(), String> {
    for (label, value) in [
        ("jméno", input.first_name.trim()), ("příjmení", input.last_name.trim()),
        ("rodné číslo", input.personal_id.trim()), ("bydliště", input.address.trim()),
        ("město", input.city.trim()), ("PSČ", input.postal_code.trim()),
        ("odborná příslušnost", input.affiliation.trim()), ("organizace / ZO", input.organization.trim()),
        ("KódOC", input.code.trim()),
    ] { if value.is_empty() { return Err(format!("Doplňte {label}.")); } }
    if !matches!(input.affiliation.as_str(), "FVČ" | "FV") { return Err("Zkontrolujte odbornou příslušnost.".into()); }
    if !matches!(input.category.as_str(), "A" | "B" | "C") { return Err("Vyberte právě jeden typ pojištění.".into()); }
    if input.annual_amount <= 0 { return Err("Vyberte právě jeden roční limit.".into()); }
    let personal = input.personal_id.trim();
    if personal.len()!=11 || personal.as_bytes().get(6)!=Some(&b'/') || !personal.chars().enumerate().all(|(i,c)| i==6 || c.is_ascii_digit()) { return Err("Zkontrolujte rodné číslo.".into()); }
    if input.postal_code.chars().filter(char::is_ascii_digit).count()!=5 || !input.postal_code.chars().all(|c|c.is_ascii_digit()||c==' ') { return Err("Zkontrolujte PSČ.".into()); }
    Ok(())
}

pub fn next_month_start(date:NaiveDate)->Option<NaiveDate>{
    let (year,month)=if date.month()==12{(date.year()+1,1)}else{(date.year(),date.month()+1)};
    NaiveDate::from_ymd_opt(year,month,1)
}

fn text(layer: &printpdf::PdfLayerReference, value: impl AsRef<str>, size: f32, x: f32, y: f32, font: &printpdf::IndirectFontRef) {
    layer.use_text(value.as_ref(), size, Mm(x), Mm(y), font);
}

fn rule(layer: &printpdf::PdfLayerReference, x1:f32,y1:f32,x2:f32,y2:f32) {
    layer.add_line(Line { points: vec![(Point::new(Mm(x1),Mm(y1)),false),(Point::new(Mm(x2),Mm(y2)),false)], is_closed:false });
}

fn outline(layer:&printpdf::PdfLayerReference,x:f32,y:f32,w:f32,h:f32){
    layer.add_line(Line{points:vec![(Point::new(Mm(x),Mm(y)),false),(Point::new(Mm(x+w),Mm(y)),false),(Point::new(Mm(x+w),Mm(y+h)),false),(Point::new(Mm(x),Mm(y+h)),false)],is_closed:true});
}

fn money(value:i64)->String { format!("{} Kč", value.to_string().as_bytes().rchunks(3).rev().map(|x|std::str::from_utf8(x).unwrap_or("")).collect::<Vec<_>>().join(" ")) }
fn cz_date(value:&str)->String { NaiveDate::parse_from_str(value,"%Y-%m-%d").map(|d|d.format("%d. %m. %Y").to_string()).unwrap_or_default() }
fn option_name(category:&str,loss:bool)->&'static str { match (category,loss) { ("B",false)=>"Standard",("B",true)=>"Standard + ztráta",("A",false)=>"Řidič",("A",true)=>"Řidič + ztráta",("C",false)=>"Strojvedoucí",("C",true)=>"Strojvedoucí + ztráta", _=>"" } }

pub fn render_pdf(data:&PdfData, tariffs:&[(String,bool,i64,i64)], destination:&Path)->Result<Vec<u8>,String> {
    let mut amounts=tariffs.iter().map(|rate|rate.2).collect::<Vec<_>>();
    amounts.sort_unstable();
    amounts.dedup();
    if amounts.is_empty(){return Err("Pro přihlášku nejsou dostupné žádné platné roční limity.".into());}
    if !data.blank&&!tariffs.iter().any(|(category,loss,amount,_)|category==&data.category&&*loss==data.loss&&*amount==data.annual_amount){return Err("Vybraný roční limit nemá platnou sazbu pro zvolenou variantu pojištění.".into());}
    let (document,page,layer)=PdfDocument::new("Přihláška k pojištění",Mm(210.0),Mm(297.0),"Přihláška");
    let regular=document.add_external_font(File::open(r"C:\Windows\Fonts\arial.ttf").map_err(|_|"Písmo Arial není dostupné.".to_string())?).map_err(|_|"Písmo Arial není dostupné.".to_string())?;
    let bold=document.add_external_font(File::open(r"C:\Windows\Fonts\arialbd.ttf").map_err(|_|"Písmo Arial není dostupné.".to_string())?).map_err(|_|"Písmo Arial není dostupné.".to_string())?;
    let l=document.get_page(page).get_layer(layer);
    l.set_outline_color(Color::Rgb(Rgb::new(0.0,0.0,0.0,None))); l.set_outline_thickness(0.35);
    text(&l,"S dokumentem je nutno nakládat ve smyslu zákona 101/2000 Sb., o ochraně osobních údajů a ve smyslu Nařízení EU č. 2016/679 o ochraně fyzických",5.4,14.0,287.0,&regular);
    text(&l,"osob (tzv. GDPR).",5.4,14.0,283.8,&regular);
    text(&l,"P ř i h l á š k a",13.0,92.0,277.0,&bold);
    text(&l,"Závazně se přihlašuji k pojištění z odpovědnosti za škody způsobené zaměstnavateli při výkonu povolání (dále",7.5,14.0,269.0,&regular);
    text(&l,format!("jen „pojištění“) ve smyslu Pojistné smlouvy {} uzavřené mezi {} a MAXIMA pojišťovna, a.s.",data.contract_number,data.federation_name),7.0,14.0,264.0,&regular);
    outline(&l,158.0,270.0,42.0,10.0); text(&l,"Evidenční číslo:",6.5,161.0,276.0,&bold); text(&l,if data.blank{""}else{&data.registration_number},8.0,182.0,273.0,&regular);
    let fields=[("Jméno a příjmení",format!("{} {}",data.first_name,data.last_name).trim().to_string()),("Rodné číslo",data.personal_id.clone()),("Bydliště",data.address.clone()),("Město",data.city.clone()),("PSČ",data.postal_code.clone()),("E-mail",data.email.clone())];
    let mut y=251.0; for (label,value) in fields { text(&l,label,8.3,14.0,y,&bold); text(&l,if data.blank{""}else{&value},8.3,46.0,y,&regular); rule(&l,44.0,y-1.0,121.0,y-1.0); y-=8.0; }
    outline(&l,134.0,211.0,66.0,42.0); text(&l,"Zaškrtněte typ pojištění a roční limit",5.7,137.0,248.0,&bold);
    let variants=[("B",false,"Standard"),("B",true,"Standard + ztráta"),("A",false,"Řidič"),("A",true,"Řidič + ztráta"),("C",false,"Strojvedoucí"),("C",true,"Strojvedoucí + ztráta")];
    text(&l,"Typ pojištění",5.6,139.0,243.0,&bold); text(&l,"Roční limit",5.6,176.0,243.0,&bold);
    for (i,(category,loss,name)) in variants.iter().enumerate(){ let yy=237.5-i as f32*4.2; let mark=if !data.blank&&data.category==*category&&data.loss==*loss{"X"}else{" "}; outline(&l,138.0,yy-1.0,3.2,3.2); text(&l,mark,5.5,139.0,yy,&bold); text(&l,*name,5.8,143.0,yy,&regular); }
    for (i,amount) in amounts.iter().enumerate(){ let yy=237.5-i as f32*4.2; let mark=if !data.blank&&data.annual_amount==*amount{"X"}else{" "}; outline(&l,175.0,yy-1.0,3.2,3.2); text(&l,mark,5.5,176.0,yy,&bold); text(&l,format!("{} tis. Kč",amount/1000),5.8,180.0,yy,&regular); }
    text(&l,"Prohlašuji, že jsem byl seznámen s pojistnými podmínkami a s roční výší pojistné částky. Zároveň s tím se zavazuji uhradit",5.9,14.0,202.0,&bold);
    text(&l,format!("pojistnou částku na účet Federace vlakových čet (číslo účtu {}, VS platby - rodné číslo bez lomítka).",data.account),5.9,14.0,198.5,&bold);
    let legal_lines=[
        "Souhlasím se shromažďováním, uchováváním a zpracováním osobních údajů obsažených v tomto formuláři správcem Federace vlakových čet, se sídlem",
        "Vinohrady (Praha 2), Wilsonova 300/8, 110 00, IČO: 43001327 pro účely pojištění. Tento souhlas uděluji pro všechny zde mnou uvedené údaje na dobu trvání",
        "pojištění, včetně stanovené doby archivace. Souhlasím s předáním svých osobních údajů v nezbytném rozsahu 3. osobě (MAXIMA pojišťovna, makléř,",
        "likvidátor) pro její vnitřní potřebu.",
        "Současně s tím jsem si vědom(a) svých práv ve smyslu § 12 a 21 zákona č. 101/2000 Sb., o ochraně osobních údajů. Se všemi vyplněnými údaji jsem byl(a)",
        "seznámen(a), údaje jsou přesné a pravdivé a byly poskytnuty dobrovolně.",
    ];
    let mut legal_y=194.5; for line in legal_lines { text(&l,line,4.6,14.0,legal_y,&regular); legal_y-=2.7; }
    text(&l,"Účel zpracování osobních údajů:",4.7,14.0,178.0,&bold);
    text(&l,"Údaje z tohoto formuláře budou použity pro vedení evidence pojištění, pro vnitřní potřebu správce a pro předání 3. osobě (MAXIMA pojišťovna, makléř,",4.6,14.0,175.3,&regular);
    text(&l,"likvidátor) pro její vnitřní potřebu.",4.6,14.0,172.6,&regular);
    text(&l,"Prohlášení správce:",4.7,14.0,169.6,&bold);
    text(&l,"Správce prohlašuje, že bude osobní údaje shromažďovat pouze v rozsahu nezbytném pro stanovený účel a zpracovávat je pouze v souladu s účelem, k němuž",4.6,14.0,166.9,&regular);
    text(&l,"byly shromážděny. Zaměstnanci správce, popř. jiné oprávněné fyzické osoby např. ve volených funkcích, které zpracovávají osobní údaje, jsou povinni",4.6,14.0,164.2,&regular);
    text(&l,"zachovávat mlčenlivost o osobních údajích, a to i po ukončení pracovního poměru, popř. ukončení činnosti ve volené funkci.",4.6,14.0,161.5,&regular);
    let displayed_date=if data.blank{"________________".to_string()}else{cz_date(&data.application_date)};
    rule(&l,14.0,156.0,95.0,156.0); rule(&l,112.0,156.0,196.0,156.0); text(&l,"Datum",5.2,50.0,152.8,&regular); text(&l,"Podpis",5.2,151.0,152.8,&regular); if !data.blank{text(&l,displayed_date,7.0,48.0,158.0,&regular)}
    text(&l,"VARIANTY POJIŠTĚNÍ",8.3,88.0,148.0,&bold); rule(&l,87.0,146.5,124.0,146.5);
    let table_variants=[("B",false,"Standard"),("A",false,"Řidič"),("C",false,"Strojvedoucí"),("B",true,"Standard + ztráta"),("A",true,"Řidič + ztráta"),("C",true,"Strojvedoucí + ztráta")];
    for (index,(category,loss,name)) in table_variants.iter().enumerate(){
        let col=index%3; let row=index/3; let x=14.0+col as f32*64.0; let top=140.0-row as f32*44.0; let w=59.0; let row_h=(24.6/amounts.len() as f32).min(4.1); let h=8.0+row_h*amounts.len() as f32;
        text(&l,format!("varianta {}",name.to_uppercase()),5.5,x+1.0,top+3.0,&bold); outline(&l,x,top-h,w,h);
        let widths=[8.0,16.0,17.0,18.0]; let mut xx=x; for width in widths.iter().take(3){xx+=*width;rule(&l,xx,top-h,xx,top);} rule(&l,x,top-8.0,x+w,top-8.0); for r in 1..amounts.len(){rule(&l,x,top-8.0-r as f32*row_h,x+w,top-8.0-r as f32*row_h);}
        text(&l,"Kat.",4.6,x+1.0,top-5.0,&bold);text(&l,"Plnění pro",4.3,x+9.0,top-3.8,&bold);text(&l,"jednu PU",4.3,x+9.0,top-6.2,&bold);text(&l,"Roční limit",4.3,x+25.0,top-5.0,&bold);text(&l,"Roční",4.3,x+42.0,top-3.8,&bold);text(&l,"pojistné",4.3,x+42.0,top-6.2,&bold);
        for (r,amount) in amounts.iter().enumerate(){let yy=top-11.3-r as f32*row_h;let price=tariffs.iter().find(|(c,z,a,_)|c==category&&z==loss&&a==amount).map(|v|v.3).unwrap_or(0);text(&l,*category,4.8,x+2.5,yy,&regular);text(&l,format!("{}",money(amount/2)),4.5,x+9.0,yy,&regular);text(&l,format!("{}",money(*amount)),4.5,x+25.0,yy,&regular);text(&l,format!("{}",money(price)),4.5,x+43.0,yy,&bold);}
    }
    let info_lines=[
        "V tabulce je uvedeno roční pojistné v šesti variantách (zvolte si pro Vás nejvhodnější variantu a zaškrtněte roční pojistnou částku). Varianta „Standard“ je",
        "základní, varianta „Řidič“ je pro osoby řídící služební motorová vozidla nebo motorové vozíky, kategorie „Strojvedoucí“ je pro osoby řídící drážní vozidla.",
        "Všechny varianty je možno zkombinovat s připojištěním „Ztráta“.",
        "Roční limit plnění si nastavte tak, aby pokryl 4,5 násobek Vaší průměrné mzdy (nejvyšší škoda, kterou může zaměstnavatel naúčtovat).",
        "• Souhrn plnění nesmí přesáhnout roční limit plnění (částka za pojistnou událost). Maximální plnění pro jednu pojistnou událost je uvedena v 2. sloupci.",
        "• Pojištěný se podílí spoluúčastí na každé pojistné události vyplývající z pojištění odpovědnosti ve výši 5 %, (resp. 10 % pro kategorii strojvedoucí) minimálně",
        "  1000 Kč. Spoluúčastí se rozumí částka, o kterou pojišťovna snižuje vyplácené plnění z každé pojistné události.",
        "• Pojištění vzniká příštím kalendářním měsícem od doručení přihlášky a připsání pojistné částky v účet prezidia FVČ.",
        "Přihlášku a úhradu pojistné částky proveďte nejpozději do 25. dne v měsíci (složenkou, příkazem k úhradě, elektronicky, hotově), podrobnější informace",
        "pro platbu si vyžádejte u předsedy ZO nebo na prezidiu FVČ. Jako variabilní symbol platby uveďte vždy své rodné číslo.",
        "S podrobnými pojistnými podmínkami se můžete seznámit u svých předsedů ZO, případně kontaktujte prezidium FVČ (972241235, 972241237).",
    ];
    let mut info_y=57.0; for line in info_lines { text(&l,line,4.7,10.5,info_y,&regular); info_y-=3.1; }
    if !data.blank { text(&l,format!("Zvoleno: {} / roční limit {} / roční pojistné {}",option_name(&data.category,data.loss),money(data.annual_amount),money(data.premium)),5.5,14.0,20.0,&bold); }
    document.save(&mut BufWriter::new(File::create(destination).map_err(|_|"PDF se nepodařilo uložit.".to_string())?)).map_err(|_|"PDF se nepodařilo vytvořit.".to_string())?;
    std::fs::read(destination).map_err(|_|"PDF se nepodařilo načíst.".to_string())
}

pub fn insert_snapshot(connection:&Connection,row_id:i64,identifier:i64,data:&PdfData,insurance_from:&str,pdf:&[u8])->rusqlite::Result<i64>{
    let hash=format!("{:x}",Sha256::digest(pdf));
    connection.execute(r#"INSERT INTO "Prihlasky"("PojistnyZaznamRowId","IdentifikatorClena","DatumPrihlasky","PojisteniOd","Kategorie","Ztrata","RocniLimit","Pojistne","Pdf","Sha256") VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"#,params![row_id,identifier,data.application_date,insurance_from,data.category,i64::from(data.loss),data.annual_amount,data.premium,pdf,hash])?;
    Ok(connection.last_insert_rowid())
}

pub fn snapshot(connection:&Connection,id:i64)->Result<(String,Vec<u8>),String>{
    connection.query_row(r#"SELECT printf('Prihlaska_%d.pdf',"IdentifikatorClena"),"Pdf" FROM "Prihlasky" WHERE "Id"=?1"#,[id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|_|"Přihlášku se nepodařilo načíst.".to_string())?.ok_or_else(||"Přihláška nebyla nalezena.".to_string())
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn insurance_starts_first_day_of_next_month(){assert_eq!(next_month_start(NaiveDate::from_ymd_opt(2026,8,10).unwrap()),NaiveDate::from_ymd_opt(2026,9,1));}
    #[test] fn insurance_start_crosses_year(){assert_eq!(next_month_start(NaiveDate::from_ymd_opt(2026,12,31).unwrap()),NaiveDate::from_ymd_opt(2027,1,1));}
    #[test] fn snapshot_is_immutable(){let c=Connection::open_in_memory().unwrap();ensure_schema(&c).unwrap();c.execute(r#"INSERT INTO "Prihlasky"("PojistnyZaznamRowId","IdentifikatorClena","DatumPrihlasky","PojisteniOd","Kategorie","Ztrata","RocniLimit","Pojistne","Pdf","Sha256")VALUES(1,1,'2026-08-10','2026-09-01','B',0,200000,495,X'25504446','x')"#,[]).unwrap();assert!(c.execute(r#"UPDATE "Prihlasky" SET "Pojistne"=1 WHERE "Id"=1"#,[]).is_err());assert!(c.execute(r#"DELETE FROM "Prihlasky" WHERE "Id"=1"#,[]).is_err());}
    #[test] fn creates_qa_pdfs_when_requested(){let Ok(dir)=std::env::var("FED_APPLICATION_PDF_QA_DIR") else{return};std::fs::create_dir_all(&dir).unwrap();let amounts=[200_000,240_000,280_000,320_000,360_000,400_000];let mut rates=Vec::new();for(category,base)in[("A",950),("B",495),("C",1901)]{for loss in[false,true]{for (i,amount)in amounts.iter().enumerate(){rates.push((category.to_string(),loss,*amount,base+i as i64*100+if loss{200}else{0}));}}}let blank=PdfData{federation_name:"Federace vlakových čet - presidium".into(),account:"400547953/0300".into(),contract_number:"650 12 00002".into(),registration_number:"".into(),first_name:"".into(),last_name:"".into(),personal_id:"".into(),address:"".into(),city:"".into(),postal_code:"".into(),email:"".into(),category:"".into(),loss:false,annual_amount:0,premium:0,application_date:"2026-08-10".into(),blank:true};render_pdf(&blank,&rates,&Path::new(&dir).join("prihlaska-prazdna.pdf")).unwrap();}
    #[test] fn creates_filled_qa_pdf_when_requested(){let Ok(dir)=std::env::var("FED_FILLED_APPLICATION_PDF_QA_DIR") else{return};std::fs::create_dir_all(&dir).unwrap();let amounts=[200_000,240_000,280_000,320_000,360_000,400_000];let mut rates=Vec::new();for(category,base)in[("A",950),("B",495),("C",1901)]{for loss in[false,true]{for (i,amount)in amounts.iter().enumerate(){rates.push((category.to_string(),loss,*amount,base+i as i64*100+if loss{200}else{0}));}}}let filled=PdfData{federation_name:"Federace vlakových čet - presidium".into(),account:"400547953/0300".into(),contract_number:"650 12 00002".into(),registration_number:"2026-001".into(),first_name:"Tereza".into(),last_name:"Nováková".into(),personal_id:"900101/1234".into(),address:"Wilsonova 300/8".into(),city:"Praha".into(),postal_code:"110 00".into(),email:"tereza@example.cz".into(),category:"A".into(),loss:true,annual_amount:320_000,premium:1_450,application_date:"2026-08-10".into(),blank:false};render_pdf(&filled,&rates,&Path::new(&dir).join("prihlaska-vyplnena.pdf")).unwrap();}
}
