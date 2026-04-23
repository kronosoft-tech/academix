//! PDF Generation Commands
//!
//! Generate real PDF files using printpdf library

use printpdf::*;
use tauri::command;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::application::use_cases::AccountingService;
use crate::infrastructure::repositories::{
    SqliteAccountCategoryRepository, SqliteAccountingEntryRepository,
};

/// Type alias for Accounting Service with SQLite repositories
/// Note: liability_repo and equity_repo are CONCRETE types, not generic
type AccountingServiceState =
    AccountingService<SqliteAccountingEntryRepository, SqliteAccountCategoryRepository>;

/// Generate Financial Balance PDF - with native save dialog
#[command]
pub async fn export_financial_balance_pdf(
    state: tauri::State<'_, AccountingServiceState>,
    app: AppHandle,
    as_of_date: String,
) -> Result<String, String> {
    println!("[PDF] Starting export for date: {}", as_of_date);

    // Get the financial balance data
    let fb = state.get_financial_balance(&as_of_date)?;
    println!(
        "[PDF] Got {} assets, {} liabilities, {} equity",
        fb.assets.len(),
        fb.liabilities.len(),
        fb.equity.len()
    );

    // Create PDF
    let (doc, page1, layer1) = PdfDocument::new(
        "Balance Financiero - Academix",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text(
        "ACADEMIX - Balance Financiero",
        20.0,
        Mm(10.0),
        Mm(280.0),
        &font,
    );
    current_layer.use_text(
        &format!("Fecha: {}", as_of_date),
        12.0,
        Mm(10.0),
        Mm(270.0),
        &font,
    );

    let mut y_pos = 255.0;

    // Assets
    current_layer.use_text("ACTIVOS", 14.0, Mm(10.0), Mm(y_pos), &font);
    y_pos -= 10.0;

    for asset in &fb.assets {
        if y_pos < 30.0 {
            break;
        }
        current_layer.use_text(
            &format!("{} - {}", asset.account_code, asset.account_name),
            10.0,
            Mm(15.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            &format!("${:.2}", asset.balance),
            10.0,
            Mm(160.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;
    }

    y_pos -= 5.0;
    current_layer.use_text(
        &format!("TOTAL ACTIVOS: ${:.2}", fb.total_assets),
        12.0,
        Mm(10.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 15.0;

    // Liabilities
    current_layer.use_text("PASIVOS", 14.0, Mm(10.0), Mm(y_pos), &font);
    y_pos -= 10.0;

    for liab in &fb.liabilities {
        if y_pos < 30.0 {
            break;
        }
        current_layer.use_text(
            &format!("{} - {}", liab.account_code, liab.account_name),
            10.0,
            Mm(15.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            &format!("${:.2}", liab.balance),
            10.0,
            Mm(160.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;
    }

    y_pos -= 5.0;
    current_layer.use_text(
        &format!("TOTAL PASIVOS: ${:.2}", fb.total_liabilities),
        12.0,
        Mm(10.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 15.0;

    // Equity
    current_layer.use_text("PATRIMONIO", 14.0, Mm(10.0), Mm(y_pos), &font);
    y_pos -= 10.0;

    for eq in &fb.equity {
        if y_pos < 30.0 {
            break;
        }
        current_layer.use_text(
            &format!("{} - {}", eq.account_code, eq.account_name),
            10.0,
            Mm(15.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            &format!("${:.2}", eq.balance),
            10.0,
            Mm(160.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;
    }

    y_pos -= 5.0;
    current_layer.use_text(
        &format!("TOTAL PATRIMONIO: ${:.2}", fb.total_equity),
        12.0,
        Mm(10.0),
        Mm(y_pos),
        &font,
    );

    current_layer.end_text_section();

    // Save to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut buffer);
        doc.save(&mut writer)
            .map_err(|e| format!("PDF save error: {}", e))?;
    }

    println!("[PDF] Generated {} bytes", buffer.len());

    // Use native save dialog
    let file_name = format!("balance-financiero-{}.pdf", as_of_date);
    let file_path = app
        .dialog()
        .file()
        .set_file_name(&file_name)
        .add_filter("PDF", &["pdf"])
        .set_title("Guardar Balance Financiero")
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string().to_string();
            std::fs::write(&path_str, &buffer).map_err(|e| format!("File write error: {}", e))?;
            println!("[PDF] Saved to: {}", path_str);
            Ok(format!("PDF guardado en: {}", path_str))
        }
        None => Err("Guardado cancelado".to_string()),
    }
}

/// Generate Income Statement PDF - with native save dialog
#[command]
pub async fn export_income_statement_pdf(
    state: tauri::State<'_, AccountingServiceState>,
    app: AppHandle,
    period_start: String,
    period_end: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    println!(
        "[PDF] Starting Income Statement export: {} to {}",
        period_start, period_end
    );

    // Get income statement data
    let is = state.get_income_statement(&period_start, &period_end)?;
    println!(
        "[PDF] Got {} income categories, {} expense categories",
        is.income_by_category.len(),
        is.expenses_by_category.len()
    );

    // Create PDF
    let (doc, page1, layer1) = PdfDocument::new(
        "Estado de Resultados - Academix",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text(
        "ACADEMIX - Estado de Resultados",
        20.0,
        Mm(10.0),
        Mm(280.0),
        &font_bold,
    );
    current_layer.use_text(
        &format!("Período: {} a {}", period_start, period_end),
        12.0,
        Mm(10.0),
        Mm(270.0),
        &font,
    );

    let mut y_pos = 255.0;

    // Income section
    current_layer.use_text("INGRESOS", 14.0, Mm(10.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;

    for cat in &is.income_by_category {
        if y_pos < 30.0 {
            break;
        }
        current_layer.use_text(&cat.category_name, 10.0, Mm(15.0), Mm(y_pos), &font);
        current_layer.use_text(
            &format!("${:.2}", cat.total),
            10.0,
            Mm(160.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;
    }

    y_pos -= 5.0;
    current_layer.use_text(
        &format!("TOTAL INGRESOS: ${:.2}", is.total_income),
        12.0,
        Mm(10.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 15.0;

    // Expenses section
    current_layer.use_text("GASTOS", 14.0, Mm(10.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;

    for cat in &is.expenses_by_category {
        if y_pos < 30.0 {
            break;
        }
        current_layer.use_text(&cat.category_name, 10.0, Mm(15.0), Mm(y_pos), &font);
        current_layer.use_text(
            &format!("${:.2}", cat.total),
            10.0,
            Mm(160.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;
    }

    y_pos -= 5.0;
    current_layer.use_text(
        &format!("TOTAL GASTOS: ${:.2}", is.total_expenses),
        12.0,
        Mm(10.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 15.0;

    // Costs section
    if is.total_costs > 0.0 {
        current_layer.use_text("COSTOS", 14.0, Mm(10.0), Mm(y_pos), &font_bold);
        y_pos -= 10.0;
        current_layer.use_text(
            &format!("TOTAL COSTOS: ${:.2}", is.total_costs),
            12.0,
            Mm(10.0),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= 15.0;
    }

    // Net Result
    let result_label = if is.is_profitable {
        "UTILIDAD NETA"
    } else {
        "PÉRDIDA NETA"
    };
    current_layer.use_text(result_label, 16.0, Mm(10.0), Mm(y_pos), &font_bold);
    current_layer.use_text(
        &format!("${:.2}", is.net_result.abs()),
        16.0,
        Mm(120.0),
        Mm(y_pos),
        &font_bold,
    );

    current_layer.end_text_section();

    // Save to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut buffer);
        doc.save(&mut writer)
            .map_err(|e| format!("PDF save error: {}", e))?;
    }

    println!("[PDF] Generated {} bytes", buffer.len());

    // Use native save dialog
    let file_name = format!("estado-resultados-{}-a-{}.pdf", period_start, period_end);
    let file_path = app
        .dialog()
        .file()
        .set_file_name(&file_name)
        .add_filter("PDF", &["pdf"])
        .set_title("Guardar Estado de Resultados")
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string().to_string();
            std::fs::write(&path_str, &buffer).map_err(|e| format!("File write error: {}", e))?;
            println!("[PDF] Saved to: {}", path_str);
            Ok(format!("PDF guardado en: {}", path_str))
        }
        None => Err("Guardado cancelado".to_string()),
    }
}
