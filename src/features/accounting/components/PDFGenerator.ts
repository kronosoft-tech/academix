// Simple HTML Report Generator - opens in browser
// Since PDF downloads are blocked in Tauri WebView

export function generateIncomeStatementHTML(report: {
  period_start: string;
  period_end: string;
  total_income: number;
  total_expenses: number;
  total_costs: number;
  net_result: number;
  is_profitable: boolean;
  income_by_category: Array<{ category_name: string; total: number }>;
  expenses_by_category: Array<{ category_name: string; total: number }>;
}): void {
  console.log("[HTML] Generating Income Statement HTML...");
  
  const formatCurrency = (amt: number) => `$${amt.toFixed(2).replace(/\d(?=(\d{3})+\.)/g, '$&,')}`;
  
  const html = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Estado de Resultados - Academix</title>
  <style>
    body { font-family: Arial, sans-serif; padding: 40px; max-width: 800px; margin: 0 auto; }
    h1 { color: #1e40af; }
    h2 { color: #374151; border-bottom: 2px solid #e5e7eb; padding-bottom: 8px; }
    table { width: 100%; border-collapse: collapse; margin: 16px 0; }
    th, td { padding: 12px; text-align: left; border-bottom: 1px solid #e5e7eb; }
    th { background: #f3f4f6; }
    .amount { text-align: right; font-family: monospace; }
    .positive { color: #16a34a; }
    .negative { color: #dc2626; }
    .total-row { font-weight: bold; background: #f9fafb; }
    .btn { background: #2563eb; color: white; padding: 12px 24px; border: none; cursor: pointer; margin-top: 20px; }
    .btn:hover { background: #1d4ed8; }
    @media print { .btn { display: none; } }
  </style>
</head>
<body>
  <h1>Academix - Estado de Resultados</h1>
  <p><strong>Período:</strong> ${report.period_start} a ${report.period_end}</p>
  
  <h2>Ingresos</h2>
  <table>
    <tr><th>Categoría</th><th class="amount">Monto</th></tr>
    ${report.income_by_category.map(c => `<tr><td>${c.category_name}</td><td class="amount">${formatCurrency(c.total)}</td></tr>`).join('')}
    <tr class="total-row"><td>TOTAL INGRESOS</td><td class="amount positive">${formatCurrency(report.total_income)}</td></tr>
  </table>
  
  <h2>Gastos</h2>
  <table>
    <tr><th>Categoría</th><th class="amount">Monto</th></tr>
    ${report.expenses_by_category.map(c => `<tr><td>${c.category_name}</td><td class="amount">${formatCurrency(c.total)}</td></tr>`).join('')}
    <tr class="total-row"><td>TOTAL GASTOS</td><td class="amount negative">${formatCurrency(report.total_expenses)}</td></tr>
  </table>
  
  <h2>Resultado</h2>
  <table>
    <tr class="total-row">
      <td>${report.is_profitable ? 'UTILIDAD' : 'PÉRDIDA'} NETA</td>
      <td class="amount ${report.is_profitable ? 'positive' : 'negative'}">${formatCurrency(Math.abs(report.net_result))}</td>
    </tr>
  </table>
  
  <button class="btn" onclick="window.print()">🖨️ Imprimir / Guardar como PDF</button>
</body>
</html>`;

  console.log("[HTML] Opening HTML report...");
  const blob = new Blob([html], { type: 'text/html' });
  const url = URL.createObjectURL(blob);
  window.open(url, '_blank');
}

export function generateFinancialBalanceHTML(report: {
  as_of_date: string;
  assets: Array<{ account_code: string; account_name: string; balance: number }>;
  liabilities: Array<{ account_code: string; account_name: string; balance: number }>;
  equity: Array<{ account_code: string; account_name: string; balance: number }>;
  total_assets: number;
  total_liabilities: number;
  total_equity: number;
}): void {
  console.log("[HTML] Generating Financial Balance HTML...");
  
  const formatCurrency = (amt: number) => `$${amt.toFixed(2).replace(/\d(?=(\d{3})+\.)/g, '$&,')}`;
  
  const html = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Balance Financiero - Academix</title>
  <style>
    body { font-family: Arial, sans-serif; padding: 40px; max-width: 800px; margin: 0 auto; }
    h1 { color: #1e40af; }
    h2 { color: #374151; border-bottom: 2px solid #e5e7eb; padding-bottom: 8px; }
    table { width: 100%; border-collapse: collapse; margin: 16px 0; }
    th, td { padding: 12px; text-align: left; border-bottom: 1px solid #e5e7eb; }
    th { background: #f3f4f6; }
    .amount { text-align: right; font-family: monospace; }
    .positive { color: #16a34a; }
    .negative { color: #dc2626; }
    .total-row { font-weight: bold; background: #f9fafb; }
    .btn { background: #2563eb; color: white; padding: 12px 24px; border: none; cursor: pointer; margin-top: 20px; }
    .btn:hover { background: #1d4ed8; }
    @media print { .btn { display: none; } }
  </style>
</head>
<body>
  <h1>Academix - Balance Financiero</h1>
  <p><strong>Fecha:</strong> ${report.as_of_date}</p>
  
  <h2>Activos</h2>
  <table>
    <tr><th>Código</th><th>Cuenta</th><th class="amount">Saldo</th></tr>
    ${report.assets.map(a => `<tr><td>${a.account_code}</td><td>${a.account_name}</td><td class="amount">${formatCurrency(a.balance)}</td></tr>`).join('')}
    <tr class="total-row"><td colspan="2">TOTAL ACTIVOS</td><td class="amount">${formatCurrency(report.total_assets)}</td></tr>
  </table>
  
  <h2>Pasivos</h2>
  <table>
    <tr><th>Código</th><th>Cuenta</th><th class="amount">Saldo</th></tr>
    ${report.liabilities.map(l => `<tr><td>${l.account_code}</td><td>${l.account_name}</td><td class="amount">${formatCurrency(l.balance)}</td></tr>`).join('')}
    <tr class="total-row"><td colspan="2">TOTAL PASIVOS</td><td class="amount">${formatCurrency(report.total_liabilities)}</td></tr>
  </table>
  
  <h2>Patrimonio</h2>
  <table>
    <tr><th>Código</th><th>Cuenta</th><th class="amount">Saldo</th></tr>
    ${report.equity.map(e => `<tr><td>${e.account_code}</td><td>${e.account_name}</td><td class="amount">${formatCurrency(e.balance)}</td></tr>`).join('')}
    <tr class="total-row"><td colspan="2">TOTAL PATRIMONIO</td><td class="amount">${formatCurrency(report.total_equity)}</td></tr>
  </table>
  
  <button class="btn" onclick="window.print()">🖨️ Imprimir / Guardar como PDF</button>
</body>
</html>`;

  console.log("[HTML] Opening HTML report...");
  const blob = new Blob([html], { type: 'text/html' });
  const url = URL.createObjectURL(blob);
  window.open(url, '_blank');
}

// Keep PDF exports for compatibility but they won't work
export function generateIncomeStatementPDF(_report: unknown) { console.log("[PDF] Disabled - use HTML version"); }
export function generateFinancialBalancePDF(_report: unknown) { console.log("[PDF] Disabled - use HTML version"); }
export function generateTrialBalancePDF(_report: unknown) {}
export function generatePayrollPDF(_report: unknown) {}
export function generateEntryReceiptPDF(_entry: unknown) {}