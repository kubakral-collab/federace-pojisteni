use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, Image, ImageTransform, Line, Mm, PdfDocument, Point, Polygon, Rgb,
};
use rusqlite::{
    params,
    types::{Value, ValueRef},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path};

const INSURER_SIGNATURE_STAMP: &[u8] = include_bytes!("../resources/insurer-signature-stamp.jpg");

pub struct InsurerPdf {
    pub policyholder: String,
    pub contract_number: String,
    pub broker: String,
    pub insurance_year: i32,
    pub rows: Vec<InsurerPdfRow>,
}
pub struct InsurerPdfRow {
    pub sequence: usize,
    pub registration_number: String,
    pub insured: String,
    pub personal_id: String,
    pub insurance_from: String,
    pub insurance_to: String,
    pub termination: String,
    pub premium: i64,
    pub unused_premium: i64,
    pub annual_limit: i64,
    pub category: String,
    pub loss: bool,
    pub note: String,
}

pub fn insurer_overview_pdf(report: &InsurerPdf, destination: &Path) -> Result<(), String> {
    let (document, first_page, first_layer) =
        PdfDocument::new("Přehled pro pojišťovnu", Mm(297.0), Mm(210.0), "Přehled");
    let regular = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbd.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let italic = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\ariali.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold_italic = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbi.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let headers = [
        "Poř.čís",
        "Evid. číslo",
        "Pojištěná osoba",
        "RČ",
        "Pojištěn Od",
        "Pojištěn Do",
        "Ukončení",
        "Roční pojistné",
        "Nespotř. pojistné",
        "Limit plnění",
        "Roční limit plnění",
        "Kategorie",
        "Poznámka",
    ];
    let widths = [
        10.3, 16.0, 40.0, 18.5, 19.0, 19.0, 18.5, 24.0, 26.0, 18.0, 26.0, 18.0, 24.9,
    ];
    let data_pages = if report.rows.len() <= 30 {
        1
    } else {
        1 + (report.rows.len() - 30 + 38) / 39
    };
    let summary = insurer_summary(report)?;
    let rows_before_last = if data_pages == 1 {
        0
    } else {
        30 + (data_pages - 2) * 39
    };
    let rows_on_last = report.rows.len().saturating_sub(rows_before_last);
    let last_rows_start_y = if data_pages == 1 { 150.5 } else { 189.5 };
    let last_rows_end_y = last_rows_start_y - rows_on_last as f32 * 4.05;
    let summary_fits_last_page = last_rows_end_y >= 114.0;
    let page_count = data_pages + usize::from(!summary_fits_last_page);
    for page_number in 0..data_pages {
        let (page, layer) = if page_number == 0 {
            (first_page, first_layer)
        } else {
            document.add_page(Mm(297.0), Mm(210.0), "Přehled")
        };
        let current = document.get_page(page).get_layer(layer);
        current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.75, None)));
        current.use_text(
            "S dokumentem je nutno nakládat ve smyslu zákona 101/2000 Sb., o ochraně osobních údajů a ve smyslu Nařízení EU č. 2016/679 o ochraně fyzických osob (tzv. GDPR) !",
            6.0, Mm(66.0), Mm(202.0), &bold_italic,
        );
        let mut y = if page_number == 0 {
            let labels = ["Pojištění klienta:", "Příloha č. 2 k PS č.:", "Makléř:"];
            let values = [
                &report.policyholder,
                &report.contract_number,
                &report.broker,
            ];
            current.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            current.set_outline_thickness(0.35);
            for i in 0..3 {
                let top = 195.0 - i as f32 * 4.2;
                current.add_polygon(Polygon {
                    rings: vec![vec![
                        (Point::new(Mm(10.0), Mm(top - 3.8)), false),
                        (Point::new(Mm(90.0), Mm(top - 3.8)), false),
                        (Point::new(Mm(90.0), Mm(top)), false),
                        (Point::new(Mm(10.0), Mm(top)), false),
                    ]],
                    mode: PaintMode::Stroke,
                    winding_order: WindingOrder::NonZero,
                });
                current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                current.use_text(labels[i], 7.3, Mm(10.2), Mm(top - 3.0), &regular);
                current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
                current.use_text(values[i], 7.3, Mm(43.5), Mm(top - 3.0), &bold);
            }
            for (i, (label, value)) in [
                (
                    "Počátek pojištění:",
                    format!("1.1.{}", report.insurance_year),
                ),
                ("Výročí PS:", format!("31.12.{}", report.insurance_year)),
            ]
            .iter()
            .enumerate()
            {
                let top = 176.5 - i as f32 * 4.2;
                current.add_polygon(Polygon {
                    rings: vec![vec![
                        (Point::new(Mm(10.0), Mm(top - 3.8)), false),
                        (Point::new(Mm(70.5), Mm(top - 3.8)), false),
                        (Point::new(Mm(70.5), Mm(top)), false),
                        (Point::new(Mm(10.0), Mm(top)), false),
                    ]],
                    mode: PaintMode::Stroke,
                    winding_order: WindingOrder::NonZero,
                });
                current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                current.use_text(*label, 7.3, Mm(10.2), Mm(top - 3.0), &regular);
                current.set_fill_color(Color::Rgb(Rgb::new(0.9, 0.0, 0.0, None)));
                current.use_text(value, 7.3, Mm(48.0), Mm(top - 3.0), &bold);
            }
            157.0
        } else {
            196.0
        };
        current.set_fill_color(Color::Rgb(Rgb::new(0.88, 0.88, 0.88, None)));
        current.add_polygon(Polygon {
            rings: vec![vec![
                (Point::new(Mm(10.8), Mm(y - 3.0)), false),
                (Point::new(Mm(289.0), Mm(y - 3.0)), false),
                (Point::new(Mm(289.0), Mm(y + 2.3)), false),
                (Point::new(Mm(10.8), Mm(y + 2.3)), false),
            ]],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        });
        current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.28, 0.62, None)));
        let mut x = 10.8;
        for (header, width) in headers.iter().zip(widths) {
            let header_width = estimated_text_width(header, 6.1);
            let header_x = x + (width - header_width).max(0.0) / 2.0;
            current.use_text(*header, 6.1, Mm(header_x), Mm(y - 1.4), &bold);
            x += width;
        }
        y -= 6.5;
        let (skip, take) = if page_number == 0 {
            (0, 30)
        } else {
            (30 + (page_number - 1) * 39, 39)
        };
        for row in report.rows.iter().skip(skip).take(take) {
            let row_top = y + 2.0;
            let cells = [
                format!("{}.", row.sequence),
                row.registration_number.clone(),
                row.insured.clone(),
                row.personal_id.clone(),
                access_date(&row.insurance_from),
                access_date(&row.insurance_to),
                access_date(&row.termination),
                access_number(row.premium),
                if row.unused_premium == 0 {
                    String::new()
                } else {
                    access_number(row.unused_premium)
                },
                if row.annual_limit >= 500_000 {
                    String::new()
                } else {
                    access_number(row.annual_limit / 2)
                },
                access_number(row.annual_limit),
                row.category.clone(),
                row.note.clone(),
            ];
            current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            let mut x = 10.8;
            for (cell, width) in cells.iter().zip(widths) {
                const CELL_PADDING: f32 = 1.1;
                let available = (width - 2.0 * CELL_PADDING).max(1.0);
                let shown = fit_text(cell, available, 6.7);
                current.use_text(shown, 6.7, Mm(x + CELL_PADDING), Mm(y - 0.55), &regular);
                x += width;
            }
            current.set_outline_color(Color::Rgb(Rgb::new(0.55, 0.55, 0.55, None)));
            current.set_outline_thickness(0.35);
            current.add_line(Line {
                points: vec![
                    (Point::new(Mm(10.8), Mm(row_top)), false),
                    (Point::new(Mm(289.0), Mm(row_top)), false),
                ],
                is_closed: false,
            });
            y -= 4.05;
        }
        let mut grid_x = 10.8;
        for width in widths {
            current.add_line(Line {
                points: vec![
                    (Point::new(Mm(grid_x), Mm(y + 4.05)), false),
                    (
                        Point::new(Mm(grid_x), Mm(if page_number == 0 { 159.3 } else { 198.3 })),
                        false,
                    ),
                ],
                is_closed: false,
            });
            grid_x += width;
        }
        current.add_line(Line {
            points: vec![
                (Point::new(Mm(289.0), Mm(y + 4.05)), false),
                (
                    Point::new(Mm(289.0), Mm(if page_number == 0 { 159.3 } else { 198.3 })),
                    false,
                ),
            ],
            is_closed: false,
        });
        if page_number + 1 == data_pages && summary_fits_last_page {
            draw_insurer_summary(
                &current,
                report,
                &summary,
                y + 1.0,
                &regular,
                &bold,
                &italic,
                &bold_italic,
            )?;
        }
        footer(&current, page_number + 1, page_count, &italic);
    }
    if !summary_fits_last_page {
        let (page, layer) = document.add_page(Mm(297.0), Mm(210.0), "Souhrn");
        let current = document.get_page(page).get_layer(layer);
        draw_insurer_summary(
            &current,
            report,
            &summary,
            196.0,
            &regular,
            &bold,
            &italic,
            &bold_italic,
        )?;
        footer(&current, page_count, page_count, &italic);
    }
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

struct InsurerSummary {
    limits: Vec<i64>,
    counts: BTreeMap<(i64, String, bool), usize>,
    premium: i64,
    unused: i64,
}

fn insurer_summary(report: &InsurerPdf) -> Result<InsurerSummary, String> {
    let mut limits = vec![200_000, 240_000, 280_000, 320_000, 360_000, 400_000];
    let mut counts = BTreeMap::new();
    for row in &report.rows {
        if !matches!(row.category.as_str(), "A" | "B" | "C") {
            return Err(format!(
                "Souhrn nelze vytvořit: neznámá kategorie {}.",
                row.category
            ));
        }
        if row.annual_limit <= 0 {
            return Err("Souhrn nelze vytvořit: chybí roční limit.".into());
        }
        limits.push(row.annual_limit);
        *counts
            .entry((row.annual_limit, row.category.clone(), row.loss))
            .or_insert(0) += 1;
    }
    limits.sort_unstable();
    limits.dedup();
    if counts.values().sum::<usize>() != report.rows.len() {
        return Err("Souhrn neobsahuje všechny exportované pojištěnce.".into());
    }
    Ok(InsurerSummary {
        limits,
        counts,
        premium: report.rows.iter().map(|row| row.premium).sum(),
        unused: report.rows.iter().map(|row| row.unused_premium).sum(),
    })
}

#[allow(clippy::too_many_arguments)]
fn draw_insurer_summary(
    layer: &printpdf::PdfLayerReference,
    report: &InsurerPdf,
    summary: &InsurerSummary,
    top: f32,
    regular: &printpdf::IndirectFontRef,
    bold: &printpdf::IndirectFontRef,
    italic: &printpdf::IndirectFontRef,
    bold_italic: &printpdf::IndirectFontRef,
) -> Result<(), String> {
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
    layer.use_text("Úhrnem počet:", 11.0, Mm(20.0), Mm(top - 4.0), regular);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.0, 0.0, None)));
    layer.use_text(
        report.rows.len().to_string(),
        11.0,
        Mm(49.0),
        Mm(top - 4.0),
        bold,
    );
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(10.0), Mm(top - 8.0)), false),
            (Point::new(Mm(289.0), Mm(top - 8.0)), false),
        ],
        is_closed: false,
    });

    let x0 = 18.0;
    let label_width = 20.0;
    let cell_width = 16.5;
    let table_width = label_width + 6.0 * cell_width;
    let category_top = top - 16.0;
    let category_bottom = category_top - 5.0;
    let table_top = category_bottom;
    let table_bottom = table_top - (summary.limits.len() as f32 + 1.0) * 5.0;

    layer.set_fill_color(Color::Rgb(Rgb::new(0.84, 0.84, 0.84, None)));
    layer.add_polygon(Polygon {
        rings: vec![vec![
            (Point::new(Mm(x0 + label_width), Mm(category_bottom)), false),
            (Point::new(Mm(x0 + table_width), Mm(category_bottom)), false),
            (Point::new(Mm(x0 + table_width), Mm(category_top)), false),
            (Point::new(Mm(x0 + label_width), Mm(category_top)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    });
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.35);
    for y in [category_top, category_bottom] {
        layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x0 + label_width), Mm(y)), false),
                (Point::new(Mm(x0 + table_width), Mm(y)), false),
            ],
            is_closed: false,
        });
    }
    for group in 0..=3 {
        let x = x0 + label_width + group as f32 * 2.0 * cell_width;
        layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x), Mm(category_bottom)), false),
                (Point::new(Mm(x), Mm(category_top)), false),
            ],
            is_closed: false,
        });
    }
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    for (group, label) in ["Ostatní zaměstnanci", "Řidiči", "Strojvedoucí"]
        .iter()
        .enumerate()
    {
        let group_x = x0 + label_width + group as f32 * 2.0 * cell_width;
        let text_width = estimated_text_width(label, 7.4);
        layer.use_text(
            *label,
            7.4,
            Mm(group_x + (2.0 * cell_width - text_width) / 2.0),
            Mm(category_top - 3.7),
            bold_italic,
        );
    }

    for row in 0..=summary.limits.len() + 1 {
        let y = table_top - row as f32 * 5.0;
        layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x0), Mm(y)), false),
                (Point::new(Mm(x0 + table_width), Mm(y)), false),
            ],
            is_closed: false,
        });
    }
    for column in 0..=7 {
        let x = if column == 0 {
            x0
        } else {
            x0 + label_width + (column - 1) as f32 * cell_width
        };
        layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x), Mm(table_bottom)), false),
                (Point::new(Mm(x), Mm(table_top)), false),
            ],
            is_closed: false,
        });
    }
    let centered_x = |value: &str, size: f32, left: f32, width: f32| {
        left + (width - estimated_text_width(value, size)).max(0.0) / 2.0
    };
    layer.use_text(
        "Roční pojistné",
        7.0,
        Mm(centered_x("Roční pojistné", 7.0, x0, label_width)),
        Mm(table_top - 3.7),
        bold_italic,
    );
    for (column, label) in [
        "Standard",
        "Se ztrátou",
        "Standard",
        "Se ztrátou",
        "Standard",
        "Se ztrátou",
    ]
    .iter()
    .enumerate()
    {
        let left = x0 + label_width + column as f32 * cell_width;
        layer.use_text(
            *label,
            6.4,
            Mm(centered_x(label, 6.4, left, cell_width)),
            Mm(table_top - 3.7),
            bold_italic,
        );
    }
    for (row_index, limit) in summary.limits.iter().enumerate() {
        let baseline = table_top - 8.7 - row_index as f32 * 5.0;
        let limit_text = access_number(*limit);
        layer.use_text(
            &limit_text,
            7.0,
            Mm(centered_x(&limit_text, 7.0, x0, label_width)),
            Mm(baseline),
            italic,
        );
        for (column, (category, loss)) in [
            ("B", false),
            ("B", true),
            ("A", false),
            ("A", true),
            ("C", false),
            ("C", true),
        ]
        .iter()
        .enumerate()
        {
            let count = summary
                .counts
                .get(&(*limit, (*category).to_string(), *loss))
                .copied()
                .unwrap_or(0);
            if count > 0 {
                let value = count.to_string();
                let left = x0 + label_width + column as f32 * cell_width;
                layer.use_text(
                    &value,
                    7.0,
                    Mm(centered_x(&value, 7.0, left, cell_width)),
                    Mm(baseline),
                    italic,
                );
            }
        }
    }

    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
    for (index, (label, value)) in [
        ("Úhrnem pojistná částka:", summary.premium),
        ("Úhrnem nespotř. pojistné:", summary.unused),
        ("Celkem k úhradě:", summary.premium - summary.unused),
    ]
    .iter()
    .enumerate()
    {
        let baseline = top - 21.0 - index as f32 * 6.0;
        layer.use_text(*label, 10.0, Mm(198.0), Mm(baseline), regular);
        let value = format!("{},00 Kč", access_number(*value));
        let value_x = 289.0 - estimated_text_width(&value, 10.0);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.0, 0.0, None)));
        layer.use_text(&value, 10.0, Mm(value_x), Mm(baseline), bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
    }

    let source = image::load_from_memory(INSURER_SIGNATURE_STAMP)
        .map_err(|_| "Razítko a podpis se nepodařilo načíst.".to_string())?;
    Image::from_dynamic_image(&source).add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(210.0)),
            translate_y: Some(Mm((top - 99.0).max(14.0))),
            dpi: Some(300.0),
            ..Default::default()
        },
    );
    Ok(())
}

fn access_number(value: i64) -> String {
    let digits = value.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    let mut value: String = out.chars().rev().collect();
    if value.starts_with('-') {
        value.insert(0, '-');
    }
    value
}

fn estimated_text_width(value: &str, font_size: f32) -> f32 {
    value
        .chars()
        .map(|character| match character {
            ' ' => 0.25,
            'i' | 'l' | 'I' | '.' | ',' | ':' => 0.24,
            'm' | 'w' | 'M' | 'W' => 0.82,
            _ => 0.60,
        })
        .sum::<f32>()
        * font_size
        * 0.352_778
}

fn fit_text(value: &str, max_width: f32, font_size: f32) -> String {
    if estimated_text_width(value, font_size) <= max_width {
        return value.to_string();
    }
    let suffix = "...";
    let mut fitted = String::new();
    for character in value.chars() {
        let candidate = format!("{fitted}{character}{suffix}");
        if estimated_text_width(&candidate, font_size) > max_width {
            break;
        }
        fitted.push(character);
    }
    format!("{fitted}{suffix}")
}
fn access_date(value: &str) -> String {
    chrono::NaiveDate::parse_from_str(value.get(..10).unwrap_or(value), "%Y-%m-%d")
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| value.to_string())
}
fn footer(
    layer: &printpdf::PdfLayerReference,
    page: usize,
    pages: usize,
    font: &printpdf::IndirectFontRef,
) {
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.65, None)));
    let date = chrono::Local::now().date_naive();
    let weekdays = [
        "pondělí",
        "terý",
        "středa",
        "čtvrtek",
        "pátek",
        "sobota",
        "neděle",
    ];
    let months = [
        "ledna",
        "února",
        "března",
        "dubna",
        "května",
        "června",
        "července",
        "srpna",
        "září",
        "října",
        "listopadu",
        "prosince",
    ];
    let stamp = format!(
        "{} {}. {} {}",
        weekdays[chrono::Datelike::weekday(&date).num_days_from_monday() as usize],
        chrono::Datelike::day(&date),
        months[chrono::Datelike::month0(&date) as usize],
        chrono::Datelike::year(&date)
    );
    layer.use_text(stamp, 6.5, Mm(10.0), Mm(14.0), font);
    layer.use_text(
        format!("Strana {page} z {pages}"),
        6.5,
        Mm(260.0),
        Mm(14.0),
        font,
    );
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFilter {
    pub kind: String,
    pub year: i32,
    pub organization_code: Option<String>,
    pub organization: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPreview {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyReportHistory {
    pub source: String,
    pub source_row_id: i64,
    pub sequence_number: Option<i64>,
    pub issued_on: Option<String>,
    pub insured_count: Option<i64>,
    pub total_amount: Option<String>,
    pub insurance_from: Option<String>,
    pub insurance_to: Option<String>,
    pub note: Option<String>,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "AuditSestav" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "Uzivatel" TEXT NOT NULL,
        "Druh" TEXT NOT NULL,
        "Format" TEXT NOT NULL,
        "PojistnyRok" INTEGER NOT NULL,
        "PocetRadku" INTEGER NOT NULL,
        "Vysledek" TEXT NOT NULL
    );"#,
    )
}

pub fn legacy_history(connection: &Connection) -> rusqlite::Result<Vec<LegacyReportHistory>> {
    let mut output = Vec::new();
    for (table, source) in [("Sestavy", "Sestava"), ("SestavyHVP", "HVP")] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )?;
        if !exists {
            continue;
        }
        let mut statement = connection.prepare(&format!(r#"SELECT rowid,* FROM "{table}""#))?;
        let rows = statement.query_map([], |row| {
            let amount = match row.get_ref(4)? {
                ValueRef::Null => None,
                other => Some(value(other)),
            };
            Ok(LegacyReportHistory {
                source: source.into(),
                source_row_id: row.get(0)?,
                sequence_number: row.get(1)?,
                issued_on: row.get(2)?,
                insured_count: row.get(3)?,
                total_amount: amount,
                insurance_from: row.get(5)?,
                insurance_to: row.get(6)?,
                note: row.get(7)?,
            })
        })?;
        output.extend(rows.collect::<rusqlite::Result<Vec<_>>>()?);
    }
    output.sort_by(|left, right| {
        right
            .issued_on
            .cmp(&left.issued_on)
            .then_with(|| right.source_row_id.cmp(&left.source_row_id))
    });
    Ok(output)
}

fn value(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(v) => v.to_string(),
        ValueRef::Real(v) => format!("{v:.2}"),
        ValueRef::Text(v) => String::from_utf8_lossy(v).into_owned(),
        ValueRef::Blob(_) => "[binární data]".into(),
    }
}

fn definition(kind: &str) -> Result<(&'static str, &'static str), String> {
    match kind {
        "insurer" => Ok((
            "Přehled pro pojišťovnu",
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Titul",'')||' '||COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rodné číslo", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Bydliště", "PSČ", "Kategorie" AS "Pojištění", "PojistnáČástka" AS "Částka", CASE WHEN "Stát"='Slovenská republika' THEN 'SK' ELSE '' END AS "Stát" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 ORDER BY "KódOC", "Příjmení", "Jméno""#,
        )),
        "oc" => Ok((
            "Sestava OC",
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rod. číslo", "PojistnáČástka" AS "Částka", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Adresa trvalého bydliště", "RočPojistné" AS "Roč. pojistné", "Kategorie" AS "Kateg." FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?2='' OR CAST("KódOC" AS TEXT)=?2) ORDER BY "EvČíslo""#,
        )),
        "zo" => Ok((
            "Sestava ZO",
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rod. číslo", "PojistnáČástka" AS "Částka", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Adresa trvalého bydliště", "RočPojistné" AS "Roč. pojistné", "Kategorie" AS "Kateg.", CASE WHEN UPPER(COALESCE("Kategorie",'')) LIKE '%Z%' THEN 'Ano' ELSE '' END AS "Ztráta", "Poznámka" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?3='' OR "ZO"=?3) ORDER BY "EvČíslo""#,
        )),
        "claims" => Ok((
            "Pojistné události / HVP",
            r#"SELECT claim."ID" AS "Poř.", TRIM(COALESCE(member."Titul",'')||' '||COALESCE(member."Příjmení",'')||' '||COALESCE(member."Jméno",'')) AS "Pojištěnec", member."EvČíslo" AS "Evid", 'Kategorie '||COALESCE(member."Kategorie",'')||'; roč. '||COALESCE(member."RočPojistné",0) AS "Typ pojištění", COALESCE(member."PojištěníOd",'')||' - '||COALESCE(member."PojištěníDo",'') AS "Pojistné období", claim."VznikPU" AS "PU ze dne", claim."ZjistenaSkoda" AS "Škoda", COALESCE(member."OdbPříslušnost",'')||CASE WHEN NULLIF(TRIM(COALESCE(member."ZO",'')),'') IS NULL THEN '' ELSE ' - ZO '||member."ZO" END AS "Odborová organizace", claim."Ukonceno" AS "Vypořádáno", claim."PopisUdalosti" AS "Popis události" FROM "PojistneUdalosti" claim JOIN "Seznam" member ON member.rowid=claim."PojistnyZaznamRowId" WHERE claim."PojistnyRok"=?1 AND (?2='' OR CAST(member."KódOC" AS TEXT)=?2) ORDER BY COALESCE(claim."PolohaVSestave", claim."ID"), claim."ID""#,
        )),
        "starts" => Ok((
            "Počátky pojištění",
            r#"SELECT "PojištěníOd" AS "Počátek", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "KódOC" AS "OC", "ZO", "Kategorie", "PojistnáČástka" AS "Pojistné" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?4='' OR date("PojištěníOd")>=date(?4)) AND (?5='' OR date("PojištěníOd")<=date(?5))"#,
        )),
        "terminations" => Ok((
            "Ukončení pojištění",
            r#"SELECT "Ukončení", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "KódOC" AS "OC", "ZO", "Kategorie" FROM "Seznam" WHERE pojisteni_rok("Ukončení")=?1 AND COALESCE("PojisteniUkonceno",0)=1 AND (?4='' OR date("Ukončení")>=date(?4)) AND (?5='' OR date("Ukončení")<=date(?5))"#,
        )),
        "contacts" => Ok((
            "Kontaktní seznam",
            r#"SELECT "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "e-mail" AS "E-mail", "Adresa", "Město", "PSČ", "ZO", "KódOC" AS "OC" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?2='' OR CAST("KódOC" AS TEXT)=?2) AND (?3='' OR "ZO"=?3)"#,
        )),
        "duplicates" => Ok((
            "Kontrola duplicit",
            r#"SELECT "RodnéČíslo" AS "Rodné číslo", COUNT(*) AS "Počet", GROUP_CONCAT("EvČíslo", ', ') AS "Evidenční čísla", GROUP_CONCAT(TRIM(COALESCE("Jméno",'')||' '||COALESCE("Příjmení",'')), '; ') AS "Jména" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND NULLIF(TRIM("RodnéČíslo"),'') IS NOT NULL GROUP BY TRIM("RodnéČíslo") HAVING COUNT(*)>1"#,
        )),
        "quality" => Ok((
            "Kontrola kvality dat",
            r#"SELECT rowid AS "Row ID", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", CASE WHEN NULLIF(TRIM("EvČíslo"),'') IS NULL THEN 'Chybí evidenční číslo' WHEN NULLIF(TRIM("RodnéČíslo"),'') IS NULL THEN 'Chybí rodné číslo' WHEN NULLIF(TRIM("Příjmení"),'') IS NULL OR NULLIF(TRIM("Jméno"),'') IS NULL THEN 'Chybí jméno' WHEN COALESCE("PojistnáČástka",0)<=0 THEN 'Chybí pojistné' WHEN COALESCE("SkutÚhrada",0)>COALESCE("PojistnáČástka",0) THEN 'Přeplatek' ELSE 'Jiný nesoulad' END AS "Problém" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (NULLIF(TRIM("EvČíslo"),'') IS NULL OR NULLIF(TRIM("RodnéČíslo"),'') IS NULL OR NULLIF(TRIM("Příjmení"),'') IS NULL OR NULLIF(TRIM("Jméno"),'') IS NULL OR COALESCE("PojistnáČástka",0)<=0 OR COALESCE("SkutÚhrada",0)>COALESCE("PojistnáČástka",0))"#,
        )),
        "member" => Ok((
            "Parametrická sestava člena",
            r#"SELECT "EvČíslo" AS "Ev. číslo","Příjmení","Jméno","RodnéČíslo" AS "Rodné číslo","KódOC" AS "OC","ZO","PojištěníOd" AS "Od","PojištěníDo" AS "Do","RočPojistné" AS "Limit","Kategorie","PojistnáČástka" AS "Pojistné","SkutÚhrada" AS "Uhrazeno","Ukončení" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?6='' OR "Příjmení" LIKE '%'||?6||'%' COLLATE NOCASE OR "Jméno" LIKE '%'||?6||'%' COLLATE NOCASE OR CAST("EvČíslo" AS TEXT)=?6 OR "RodnéČíslo" LIKE '%'||?6||'%')"#,
        )),
        _ => Err("Neznámý druh sestavy.".into()),
    }
}

pub fn preview(connection: &Connection, filter: &ReportFilter) -> Result<ReportPreview, String> {
    let (title, sql) = definition(&filter.kind)?;
    let oc = filter.organization_code.as_deref().unwrap_or_default();
    let org = filter.organization.as_deref().unwrap_or_default();
    let from = filter.date_from.as_deref().unwrap_or_default();
    let to = filter.date_to.as_deref().unwrap_or_default();
    let search = filter.search.as_deref().unwrap_or_default();
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| "Sestavu se nepodařilo připravit.".to_string())?;
    let columns = statement
        .column_names()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    let parameter_count = statement.parameter_count();
    let values = vec![
        Value::Integer(filter.year as i64),
        Value::Text(oc.into()),
        Value::Text(org.into()),
        Value::Text(from.into()),
        Value::Text(to.into()),
        Value::Text(search.into()),
    ];
    let rows = statement
        .query_map(
            rusqlite::params_from_iter(values.into_iter().take(parameter_count)),
            |row| {
                (0..columns.len())
                    .map(|index| row.get_ref(index).map(value))
                    .collect::<rusqlite::Result<Vec<_>>>()
            },
        )
        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<_>>>())
        .map_err(|_| "Sestavu se nepodařilo načíst.".to_string())?;
    Ok(ReportPreview {
        title: title.into(),
        columns,
        total_rows: rows.len(),
        rows,
    })
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub fn csv(report: &ReportPreview) -> Vec<u8> {
    let mut output = String::from("\u{feff}");
    output.push_str(
        &report
            .columns
            .iter()
            .map(|v| csv_escape(v))
            .collect::<Vec<_>>()
            .join(";"),
    );
    output.push_str("\r\n");
    for row in &report.rows {
        output.push_str(
            &row.iter()
                .map(|v| csv_escape(v))
                .collect::<Vec<_>>()
                .join(";"),
        );
        output.push_str("\r\n");
    }
    output.into_bytes()
}

pub fn pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    if report.title == "Pojistné události / HVP" {
        return claims_pdf(report, destination);
    }
    if matches!(
        report.title.as_str(),
        "Sestava OC" | "Sestava ZO" | "Přehled pro pojišťovnu"
    ) {
        return access_table_pdf(report, destination);
    }
    let (document, page, layer) = PdfDocument::new(&report.title, Mm(297.0), Mm(210.0), "Sestava");
    let font = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let mut current_page = page;
    let mut current_layer = layer;
    let mut y = 198.0;
    for (index, line) in std::iter::once(report.title.clone())
        .chain(std::iter::once(report.columns.join(" | ")))
        .chain(report.rows.iter().map(|r| r.join(" | ")))
        .enumerate()
    {
        if y < 12.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            current_page = next.0;
            current_layer = next.1;
            y = 198.0;
        }
        let text: String = line.chars().take(180).collect();
        document
            .get_page(current_page)
            .get_layer(current_layer)
            .use_text(
                text,
                if index == 0 { 14.0 } else { 7.0 },
                Mm(8.0),
                Mm(y),
                &font,
            );
        y -= if index == 0 { 10.0 } else { 5.0 };
    }
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

fn report_column_widths(title: &str) -> &'static [f32] {
    match title {
        "Sestava OC" => &[18.0, 39.0, 27.0, 20.0, 75.0, 25.0, 18.0],
        "Sestava ZO" => &[17.0, 36.0, 25.0, 19.0, 59.0, 23.0, 16.0, 16.0, 28.0],
        _ => &[19.0, 45.0, 29.0, 72.0, 17.0, 24.0, 22.0, 12.0],
    }
}

fn access_table_pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    let (document, first_page, first_layer) =
        PdfDocument::new(&report.title, Mm(297.0), Mm(210.0), "Sestava");
    let regular = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbd.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let widths = report_column_widths(&report.title);
    let mut page = first_page;
    let mut layer = first_layer;
    let mut page_number = 1;
    let mut y = 178.0;
    let draw_header = |page, layer, page_number: usize| {
        let current = document.get_page(page).get_layer(layer);
        current.use_text(
            "S dokumentem je nutno nakládat v souladu s pravidly ochrany osobních údajů.",
            6.0,
            Mm(64.0),
            Mm(204.0),
            &regular,
        );
        current.use_text(report.title.to_uppercase(), 16.0, Mm(7.0), Mm(194.0), &bold);
        let mut x = 7.0;
        for (index, column) in report.columns.iter().enumerate() {
            let width = widths.get(index).copied().unwrap_or(25.0);
            current.use_text(
                clipped(column, (width / 2.0).max(5.0) as usize),
                7.0,
                Mm(x),
                Mm(184.0),
                &bold,
            );
            x += width;
        }
        current.use_text(
            format!("Strana {page_number}"),
            7.0,
            Mm(270.0),
            Mm(7.0),
            &regular,
        );
    };
    draw_header(page, layer, page_number);
    for row in &report.rows {
        if y < 18.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            page = next.0;
            layer = next.1;
            page_number += 1;
            y = 178.0;
            draw_header(page, layer, page_number);
        }
        let current = document.get_page(page).get_layer(layer);
        let mut x = 7.0;
        for (index, value) in row.iter().enumerate() {
            let width = widths.get(index).copied().unwrap_or(25.0);
            current.use_text(
                clipped(value, (width / 1.7).max(5.0) as usize),
                7.0,
                Mm(x),
                Mm(y),
                &regular,
            );
            x += width;
        }
        y -= 5.0;
    }
    let current = document.get_page(page).get_layer(layer);
    current.use_text(
        format!("Úhrnem počet: {}", report.total_rows),
        9.0,
        Mm(7.0),
        Mm(y.max(12.0)),
        &bold,
    );
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

fn clipped(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let text: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{text}...")
    } else {
        text
    }
}

fn claims_pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    let (document, first_page, first_layer) = PdfDocument::new(
        "SESTAVA POJISTNÝCH UDÁLOSTÍ",
        Mm(297.0),
        Mm(210.0),
        "Sestava",
    );
    let regular = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbd.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let italic = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\ariali.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold_italic = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbi.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let mut page = first_page;
    let mut layer = first_layer;
    let mut y = 183.0;
    let mut page_number = 1;
    let draw_header = |page, layer, page_number: usize| {
        let current = document.get_page(page).get_layer(layer);
        current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.36, 0.72, None)));
        current.use_text(
            "S dokumentem je nutno nakládat ve smyslu zákona 101/2000 Sb., o ochraně osobních údajů a ve smyslu Nařízení EU č. 2016/679 o ochraně fyzických osob (tzv. GDPR) !",
            5.5,
            Mm(64.0),
            Mm(204.0),
            &bold_italic,
        );
        current.set_fill_color(Color::Rgb(Rgb::new(0.72, 0.0, 0.0, None)));
        current.use_text(
            "SESTAVA POJISTNÝCH UDÁLOSTÍ",
            16.0,
            Mm(7.0),
            Mm(194.0),
            &bold_italic,
        );
        current.set_outline_color(Color::Rgb(Rgb::new(0.72, 0.0, 0.0, None)));
        current.set_outline_thickness(0.5);
        current.add_line(Line {
            points: vec![
                (Point::new(Mm(7.0), Mm(192.8)), false),
                (Point::new(Mm(92.0), Mm(192.8)), false),
            ],
            is_closed: false,
        });
        current.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
        current.use_text(
            "Federace vlakových čet - presidium, Wilsonova 300/8, 110 00 Vinohrady (Praha 2)",
            6.5,
            Mm(100.0),
            Mm(7.0),
            &italic,
        );
        let _ = page_number;
    };
    draw_header(page, layer, page_number);
    for row in &report.rows {
        if y < 34.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            page = next.0;
            layer = next.1;
            page_number += 1;
            y = 183.0;
            draw_header(page, layer, page_number);
        }
        let current = document.get_page(page).get_layer(layer);
        let at = |index: usize| row.get(index).map(String::as_str).unwrap_or("");
        for (bottom, height, shade) in [(y - 0.5, 4.2, 0.92), (y - 4.7, 4.2, 0.92)] {
            current.set_fill_color(Color::Rgb(Rgb::new(shade, shade, shade, None)));
            current.add_polygon(Polygon {
                rings: vec![vec![
                    (Point::new(Mm(7.0), Mm(bottom)), false),
                    (Point::new(Mm(290.0), Mm(bottom)), false),
                    (Point::new(Mm(290.0), Mm(bottom + height)), false),
                    (Point::new(Mm(7.0), Mm(bottom + height)), false),
                ]],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::NonZero,
            });
        }
        let columns = [7.5, 14.0, 67.0, 78.0, 126.0, 171.0, 190.0, 221.0, 267.0];
        let headers = [
            "Poř.:",
            "Pojištěnec:",
            "Evid:",
            "Typ pojištění",
            "Pojistné období",
            "PU ze dne",
            "Škoda",
            "Odborová organizace",
            "Vypořádáno",
        ];
        let cells = [
            at(0),
            at(1),
            at(2),
            at(3),
            at(4),
            at(5),
            at(6),
            at(7),
            at(8),
        ];
        current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.85, None)));
        for (x, header) in columns.iter().zip(headers.iter()) {
            current.use_text(*header, 6.0, Mm(*x), Mm(y + 0.7), &bold);
        }
        current.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        for (index, (x, cell)) in columns.iter().zip(cells.iter()).enumerate() {
            let shown = if index == 3 {
                clipped(cell, 27)
            } else if index == 7 {
                clipped(cell, 25)
            } else {
                clipped(cell, 20)
            };
            current.use_text(
                shown,
                6.2,
                Mm(*x),
                Mm(y - 3.6),
                if index < 2 { &bold } else { &regular },
            );
        }
        if !at(8).trim().is_empty() {
            current.set_fill_color(Color::Rgb(Rgb::new(1.0, 0.0, 0.0, None)));
            current.use_text(at(8), 6.2, Mm(columns[8]), Mm(y - 3.6), &bold);
        }
        y -= 9.0;
        current.set_fill_color(Color::Rgb(Rgb::new(0.12, 0.12, 0.12, None)));
        current.use_text("Popis události:", 6.5, Mm(7.0), Mm(y), &bold_italic);
        current.add_line(Line {
            points: vec![
                (Point::new(Mm(7.0), Mm(y - 0.6)), false),
                (Point::new(Mm(29.0), Mm(y - 0.6)), false),
            ],
            is_closed: false,
        });
        y -= 4.0;
        let description = at(9).replace(['\r', '\n'], " ");
        let characters: Vec<char> = description.chars().collect();
        for chunk in characters.chunks(180).take(3) {
            current.set_fill_color(Color::Rgb(Rgb::new(0.58, 0.58, 0.58, None)));
            current.use_text(
                chunk.iter().collect::<String>(),
                6.8,
                Mm(7.0),
                Mm(y),
                &italic,
            );
            y -= 4.0;
        }
        current.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        current.set_outline_thickness(0.5);
        current.add_line(Line {
            points: vec![
                (Point::new(Mm(7.0), Mm(y - 1.0)), false),
                (Point::new(Mm(290.0), Mm(y - 1.0)), false),
            ],
            is_closed: false,
        });
        y -= 6.0;
    }
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

pub fn audit(
    connection: &Connection,
    user: &str,
    filter: &ReportFilter,
    format: &str,
    count: usize,
) {
    let _=connection.execute(r#"INSERT INTO "AuditSestav"("Uzivatel","Druh","Format","PojistnyRok","PocetRadku","Vysledek") VALUES(?1,?2,?3,?4,?5,'OK')"#, params![user,filter.kind,format,filter.year,count as i64]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_quotes_semicolons_and_quotes() {
        let report = ReportPreview {
            title: "Test".into(),
            columns: vec!["A".into()],
            rows: vec![vec!["x;y\"z".into()]],
            total_rows: 1,
        };
        let output = String::from_utf8(csv(&report)).unwrap();
        assert!(output.contains("\"x;y\"\"z\""));
    }

    #[test]
    fn claims_pdf_uses_access_style_grouped_layout() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("claims-report.pdf"))
            .unwrap_or_else(|| directory.path().join("claims.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let report = ReportPreview {
            title: "Pojistné události / HVP".into(),
            columns: vec![
                "Poř.".into(),
                "Pojištěnec".into(),
                "Evid".into(),
                "Typ pojištění".into(),
                "Pojistné období".into(),
                "PU ze dne".into(),
                "Škoda".into(),
                "Odborová organizace".into(),
                "Vypořádáno".into(),
                "Popis události".into(),
            ],
            rows: vec![vec![
                "1".into(),
                "Jan Novák".into(),
                "10001".into(),
                "Kategorie A; roč. 320000".into(),
                "01.01.2024 - 31.12.2024".into(),
                "10.06.2024".into(),
                "15000".into(),
                "FVČ - ZO PRAHA".into(),
                "".into(),
                "Popis testovací události".into(),
            ]],
            total_rows: 1,
        };
        pdf(&report, &destination).unwrap();
        let bytes = std::fs::read(destination).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 1_000);
    }

    #[test]
    fn access_table_pdf_has_headers_and_rows() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("oc-report.pdf"))
            .unwrap_or_else(|| directory.path().join("oc.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let report = ReportPreview {
            title: "Sestava OC".into(),
            columns: vec![
                "Evid. číslo".into(),
                "Příjmení a jméno".into(),
                "Rod. číslo".into(),
                "Částka".into(),
                "Adresa trvalého bydliště".into(),
                "Roč. pojistné".into(),
                "Kateg.".into(),
            ],
            rows: vec![vec![
                "10001".into(),
                "Jan Novák".into(),
                "800101/0000".into(),
                "1500".into(),
                "Hlavní 1, Praha".into(),
                "320000".into(),
                "A".into(),
            ]],
            total_rows: 1,
        };
        pdf(&report, &destination).unwrap();
        assert!(std::fs::metadata(destination).unwrap().len() > 1_000);
    }

    #[test]
    fn insurer_overview_pdf_keeps_all_750_rows_and_paginates() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("Prehled_pro_pojistovnu_2027.pdf"))
            .unwrap_or_else(|| directory.path().join("Prehled_pro_pojistovnu_2027.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let rows = (1..=750)
            .map(|sequence| InsurerPdfRow {
                sequence,
                registration_number: sequence.to_string(),
                insured: if sequence % 37 == 0 {
                    format!("Mgr. Dlouhopříjmeníová Alexandra {sequence}")
                } else if sequence % 11 == 0 {
                    format!("Šťastný Jiří {sequence}")
                } else {
                    format!("Novák Jan {sequence}")
                },
                personal_id: "800101/0000".into(),
                insurance_from: "2027-01-01".into(),
                insurance_to: "2027-12-31".into(),
                termination: String::new(),
                premium: if sequence % 17 == 0 { 12_345 } else { 700 },
                unused_premium: if sequence % 41 == 0 { 1_250 } else { 0 },
                annual_limit: if sequence % 13 == 0 { 600_000 } else { 200_000 },
                category: if sequence % 13 == 0 {
                    "C".into()
                } else {
                    "B".into()
                },
                loss: sequence % 2 == 0 || sequence % 37 == 0,
                note: if sequence % 37 == 0 {
                    "Poj. na ztrátu; delší testovací poznámka".into()
                } else if sequence % 2 == 0 {
                    "Poj. na ztrátu".into()
                } else {
                    String::new()
                },
            })
            .collect();
        insurer_overview_pdf(
            &InsurerPdf {
                policyholder: "Federace vlakových čet".into(),
                contract_number: "650 12 00002".into(),
                broker: "Abaccus finance s.r.o.".into(),
                insurance_year: 2027,
                rows,
            },
            &destination,
        )
        .unwrap();
        let document = lopdf::Document::load(&destination).unwrap();
        assert_eq!(document.get_pages().len(), 20);
        assert!(std::fs::metadata(destination).unwrap().len() > 10_000);
    }

    #[test]
    fn insurer_overview_summary_is_data_driven_and_paginates_safely() {
        let row =
            |sequence: usize, limit: i64, category: &str, loss: bool, premium: i64| InsurerPdfRow {
                sequence,
                registration_number: (10_000 + sequence).to_string(),
                insured: format!("Testovací člen {sequence}"),
                personal_id: "800101/0000".into(),
                insurance_from: "2026-04-01".into(),
                insurance_to: "2026-12-31".into(),
                termination: String::new(),
                premium,
                unused_premium: 0,
                annual_limit: limit,
                category: category.into(),
                loss,
                note: if loss {
                    "Poj. na ztrátu".into()
                } else {
                    String::new()
                },
            };
        let rows = vec![
            row(1, 280_000, "B", true, 613),
            row(2, 240_000, "B", true, 708),
            row(3, 240_000, "B", true, 531),
        ];
        let report = InsurerPdf {
            policyholder: "Federace vlakových čet".into(),
            contract_number: "650 12 00002".into(),
            broker: "Abaccus finance s.r.o.".into(),
            insurance_year: 2026,
            rows,
        };
        let summary = insurer_summary(&report).unwrap();
        assert_eq!(summary.counts.values().sum::<usize>(), 3);
        assert_eq!(summary.counts[&(240_000, "B".into(), true)], 2);
        assert_eq!(summary.counts[&(280_000, "B".into(), true)], 1);
        assert_eq!(summary.premium, 1_852);
        assert_eq!(summary.unused, 0);

        let directory = tempfile::tempdir().unwrap();
        let qa_directory = std::env::var_os("FED_PDF_QA_DIR").map(std::path::PathBuf::from);
        let three_path = qa_directory
            .as_ref()
            .map(|path| path.join("Prehled_pro_pojistovnu_2026_3_clenove.pdf"))
            .unwrap_or_else(|| directory.path().join("three.pdf"));
        insurer_overview_pdf(&report, &three_path).unwrap();
        assert_eq!(
            lopdf::Document::load(&three_path)
                .unwrap()
                .get_pages()
                .len(),
            1
        );

        let thirty_rows = (1..=30)
            .map(|sequence| row(sequence, 240_000, "B", sequence % 2 == 0, 700))
            .collect();
        let thirty_report = InsurerPdf {
            rows: thirty_rows,
            ..report
        };
        let thirty_path = qa_directory
            .as_ref()
            .map(|path| path.join("Prehled_pro_pojistovnu_2026_30_clenu.pdf"))
            .unwrap_or_else(|| directory.path().join("thirty.pdf"));
        insurer_overview_pdf(&thirty_report, &thirty_path).unwrap();
        assert_eq!(
            lopdf::Document::load(&thirty_path)
                .unwrap()
                .get_pages()
                .len(),
            2
        );
    }

    #[test]
    fn insurer_cell_text_respects_inner_width_including_ellipsis() {
        for width in [8.5, 16.0, 33.5] {
            let shown = fit_text("Poj. na ztrátu; mimořádně dlouhá poznámka", width, 6.7);
            assert!(estimated_text_width(&shown, 6.7) <= width);
        }
    }

    #[test]
    fn legacy_report_history_reads_both_access_tables_by_position() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(r#"
            CREATE TABLE "Sestavy" ("a" INTEGER,"b" TEXT,"c" INTEGER,"d" NUMERIC,"e" TEXT,"f" TEXT,"g" TEXT);
            CREATE TABLE "SestavyHVP" ("a" INTEGER,"b" TEXT,"c" INTEGER,"d" NUMERIC,"e" TEXT,"f" TEXT,"g" TEXT);
            INSERT INTO "Sestavy" VALUES(NULL,'2024-03-07 00:00:00',7,0,NULL,'2024-03-31 00:00:00',NULL);
            INSERT INTO "SestavyHVP" VALUES(1,'2002-01-09 00:00:00',25,5570,'2002-01-01 00:00:00',NULL,NULL);
        "#).unwrap();
        let rows = legacy_history(&connection).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source, "Sestava");
        assert_eq!(rows[1].source, "HVP");
        assert_eq!(rows[1].insured_count, Some(25));
        assert_eq!(rows[1].total_amount.as_deref(), Some("5570"));
    }

    #[test]
    fn verified_access_report_history_contains_141_rows() {
        let Some(path) = std::env::var_os("FED_REPORT_HISTORY_DB") else {
            return;
        };
        let connection =
            Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let rows = legacy_history(&connection).unwrap();
        assert_eq!(rows.len(), 141);
        assert_eq!(rows.iter().filter(|row| row.source == "Sestava").count(), 3);
        assert_eq!(rows.iter().filter(|row| row.source == "HVP").count(), 138);
    }
}
