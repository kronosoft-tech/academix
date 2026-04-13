// PDF Generator - Phase 11
// Generate PDF reports using jsPDF

import jsPDF from "jspdf";
import autoTable from "jspdf-autotable";

// Types for PDF data
interface TrialBalanceReport {
  as_of_date: string;
  accounts: Array<{
    account_code: string;
    account_name: string;
    debit_balance: number;
    credit_balance: number;
    balance_type: string;
  }>;
  total_debits: number;
  total_credits: number;
  is_balanced: boolean;
}

interface IncomeStatementReport {
  period_start: string;
  period_end: string;
  total_income: number;
  total_expenses: number;
  total_costs: number;
  net_result: number;
  is_profitable: boolean;
  income_by_category: Array<{ category_name: string; total: number }>;
  expenses_by_category: Array<{ category_name: string; total: number }>;
}

interface PayrollReport {
  run: {
    period_display: string;
    created_at: string;
    total_gross: number;
    total_deductions: number;
    total_net: number;
  };
  entries: Array<{
    employee_name: string;
    gross_income: number;
    total_deductions: number;
    net_income: number;
  }>;
}

// Company header for all reports
function addHeader(doc: jsPDF, title: string, subtitle?: string) {
  // Company name
  doc.setFontSize(18);
  doc.setTextColor(40, 40, 40);
  doc.text("Academix", 14, 20);

  // Report title
  doc.setFontSize(14);
  doc.setTextColor(60, 60, 60);
  doc.text(title, 14, 30);

  // Subtitle (date range)
  if (subtitle) {
    doc.setFontSize(10);
    doc.setTextColor(100, 100, 100);
    doc.text(subtitle, 14, 38);
  }

  // Date generated
  doc.setFontSize(9);
  doc.setTextColor(150, 150, 150);
  doc.text(`Generado: ${new Date().toLocaleDateString("es-PE")}`, 14, 45);
}

// Generate Trial Balance PDF
export function generateTrialBalancePDF(report: TrialBalanceReport): void {
  const doc = new jsPDF();
  
  addHeader(doc, "Balance de Comprobación", `Al ${report.as_of_date}`);
  
  // Table data
  const tableData = report.accounts.map((acc) => [
    acc.account_code,
    acc.account_name,
    acc.debit_balance > 0 ? `S/ ${acc.debit_balance.toFixed(2)}` : "-",
    acc.credit_balance > 0 ? `S/ ${acc.credit_balance.toFixed(2)}` : "-",
  ]);
  
  // Add totals row
  tableData.push([
    "",
    "TOTALES",
    `S/ ${report.total_debits.toFixed(2)}`,
    `S/ ${report.total_credits.toFixed(2)}`,
  ]);
  
  autoTable(doc, {
    startY: 55,
    head: [["Código", "Cuenta", "Débitos", "Créditos"]],
    body: tableData,
    theme: "striped",
    headStyles: {
      fillColor: [59, 130, 246],
      textColor: 255,
      fontStyle: "bold",
    },
    styles: {
      fontSize: 9,
      cellPadding: 3,
    },
    columnStyles: {
      0: { cellWidth: 25 },
      1: { cellWidth: "auto" },
      2: { cellWidth: 30, halign: "right" },
      3: { cellWidth: 30, halign: "right" },
    },
    foot: [[{ content: report.is_balanced ? "✓ Balance correcto" : "✗ ERROR: Desbalanceado", colSpan: 4, styles: { fontStyle: "bold", textColor: report.is_balanced ? [34, 197, 94] : [239, 68, 68] } }]],
  });
  
  doc.save(`balance-comprobacion-${report.as_of_date}.pdf`);
}

// Generate Income Statement PDF
export function generateIncomeStatementPDF(report: IncomeStatementReport): void {
  const doc = new jsPDF();
  
  const dateRange = `${report.period_start} - ${report.period_end}`;
  addHeader(doc, "Estado de Resultados", dateRange);
  
  // Summary section
  const summaryY = 55;
  
  doc.setFontSize(12);
  doc.setTextColor(40, 40, 40);
  doc.text("Resumen", 14, summaryY);
  
  autoTable(doc, {
    startY: summaryY + 5,
    head: [["Concepto", "Monto"]],
    body: [
      ["Total Ingresos", `S/ ${report.total_income.toFixed(2)}`],
      ["Total Gastos", `S/ ${report.total_expenses.toFixed(2)}`],
      ["Total Costos", `S/ ${report.total_costs.toFixed(2)}`],
      ["RESULTADO DEL EJERCICIO", `S/ ${report.net_result.toFixed(2)}`],
    ],
    theme: "plain",
    styles: {
      fontSize: 10,
      cellPadding: 4,
    },
    columnStyles: {
      0: { fontStyle: "bold" },
      1: { halign: "right" },
    },
    foot: [[{ content: report.is_profitable ? "RESULTADO: UTILIDAD" : "RESULTADO: PÉRDIDA", colSpan: 2, styles: { fontStyle: "bold", fillColor: report.is_profitable ? [220, 252, 231] : [254, 226, 226], textColor: report.is_profitable ? [22, 101, 52] : [153, 27, 27] } }]],
  });
  
  // Income by category
  const incomeY = (doc as unknown as { lastAutoTable: { finalY: number } }).lastAutoTable.finalY + 15;
  
  doc.setFontSize(12);
  doc.setTextColor(40, 40, 40);
  doc.text("Ingresos por Categoría", 14, incomeY);
  
  autoTable(doc, {
    startY: incomeY + 5,
    head: [["Categoría", "Total"]],
    body: report.income_by_category.map((c) => [c.category_name, `S/ ${c.total.toFixed(2)}`]),
    theme: "striped",
    headStyles: { fillColor: [34, 197, 94] },
    styles: { fontSize: 9 },
    columnStyles: { 1: { halign: "right" } },
  });
  
  // Expenses by category
  const expenseY = (doc as unknown as { lastAutoTable: { finalY: number } }).lastAutoTable.finalY + 15;
  
  doc.setFontSize(12);
  doc.setTextColor(40, 40, 40);
  doc.text("Gastos por Categoría", 14, expenseY);
  
  autoTable(doc, {
    startY: expenseY + 5,
    head: [["Categoría", "Total"]],
    body: report.expenses_by_category.map((c) => [c.category_name, `S/ ${c.total.toFixed(2)}`]),
    theme: "striped",
    headStyles: { fillColor: [239, 68, 68] },
    styles: { fontSize: 9 },
    columnStyles: { 1: { halign: "right" } },
  });
  
  doc.save(`estado-resultados-${report.period_start}.pdf`);
}

// Generate Payroll PDF
export function generatePayrollPDF(report: PayrollReport): void {
  const doc = new jsPDF();
  
  addHeader(doc, "Planilla de Nómina", report.run.period_display);
  
  // Summary
  autoTable(doc, {
    startY: 55,
    head: [["Concepto", "Monto"]],
    body: [
      ["Total Bruto", `S/ ${report.run.total_gross.toFixed(2)}`],
      ["Total Deducciones", `S/ ${report.run.total_deductions.toFixed(2)}`],
      ["Total Neto", `S/ ${report.run.total_net.toFixed(2)}`],
    ],
    theme: "plain",
    styles: { fontSize: 10, cellPadding: 4 },
    columnStyles: { 1: { halign: "right" } },
  });
  
  // Employee entries
  autoTable(doc, {
    startY: (doc as unknown as { lastAutoTable: { finalY: number } }).lastAutoTable.finalY + 15,
    head: [["Empleado", "Bruto", "Deducciones", "Neto"]],
    body: report.entries.map((e) => [
      e.employee_name,
      `S/ ${e.gross_income.toFixed(2)}`,
      `S/ ${e.total_deductions.toFixed(2)}`,
      `S/ ${e.net_income.toFixed(2)}`,
    ]),
    theme: "striped",
    headStyles: { fillColor: [6, 182, 212] },
    styles: { fontSize: 9, cellPadding: 3 },
    columnStyles: {
      0: { cellWidth: 60 },
      1: { halign: "right" },
      2: { halign: "right" },
      3: { halign: "right", fontStyle: "bold" },
    },
  });
  
  doc.save(`planilla-nomina-${report.run.period_display.replace(" ", "-")}.pdf`);
}

// Generate Accounting Entry Receipt
export function generateEntryReceiptPDF(entry: {
  reference: string;
  date: string;
  description: string;
  debit_account: string;
  credit_account: string;
  amount: number;
}): void {
  const doc = new jsPDF();
  
  // Header
  doc.setFontSize(18);
  doc.setTextColor(40, 40, 40);
  doc.text("Comprobante Contable", 14, 20);
  
  doc.setFontSize(12);
  doc.setTextColor(60, 60, 60);
  doc.text(`Referencia: ${entry.reference}`, 14, 30);
  doc.text(`Fecha: ${entry.date}`, 14, 37);
  
  // Entry details
  autoTable(doc, {
    startY: 45,
    head: [["Concepto", "Detalle"]],
    body: [
      ["Descripción", entry.description],
      ["Cuenta Debe", entry.debit_account],
      ["Cuenta Haber", entry.credit_account],
      ["Monto", `S/ ${entry.amount.toFixed(2)}`],
    ],
    theme: "plain",
    styles: { fontSize: 10, cellPadding: 4 },
  });
  
  // Signature section
  doc.setFontSize(9);
  doc.setTextColor(150, 150, 150);
  doc.text("_________________________", 14, 80);
  doc.text("Firma del responsable", 14, 85);
  
  doc.save(`comprobante-${entry.reference}.pdf`);
}

// Export all generators
export const PDFGenerators = {
  trialBalance: generateTrialBalancePDF,
  incomeStatement: generateIncomeStatementPDF,
  payroll: generatePayrollPDF,
  entryReceipt: generateEntryReceiptPDF,
};

export default PDFGenerators;