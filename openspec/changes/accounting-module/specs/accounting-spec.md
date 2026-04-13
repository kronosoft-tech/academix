# Módulo de Contabilidad - Specification (Deep)

## Propósito
Este documento especifica los requisitos para el Módulo de Contabilidad de Academix, incluyendo gestión de nómina (payroll), facturación, reportes financieros y gestión de empleados.

---

## 1. Gestión de Empleados (Employee Management)

### Requisito: EM-001 - Registro de Empleados
El sistema DEBE permitir registrar empleados con datos laborales completos.

#### Escenario: Registro básico de empleado
- GIVEN el usuario accede al formulario de nuevo empleado
- WHEN completa los campos: nombre, identificación, correo, teléfono, dirección, cargo, departamento, fecha de ingreso, tipo de contrato, salary base
- AND guarda el formulario
- THEN el sistema CREA el registro del empleado
- AND asigna un ID único
- AND muestra mensaje de confirmación

#### Escenario: Registro con datos incompletos
- GIVEN el usuario intenta guardar empleado con campos obligatorios vacíos
- WHEN intenta guardar
- THEN el sistema MUESTRA errores de validación
- AND NO permite guardar hasta completar campos obligatorios

### Requisito: EM-002 - Tipos de Contrato
El sistema DEBE soportar múltiples tipos de contrato.

- **Contrato Fijo**: Duración definida, salario fijo mensual
- **Contrato Indefinido**: Sin fecha de terminación, salario fijo mensual
- **Contrato por Horas**: Salario basado en horas trabajadas
- **Contrato por Prestación de Servicios**: Factura por servicio

#### Escenario: Cambio de tipo de contrato
- GIVEN un empleado tiene contrato fijo
- WHEN el administrador cambia el tipo a indefinido
- THEN el sistema ACTUALIZA el tipo de contrato
- AND mantiene el historial de contratos anteriores

### Requisito: EM-003 - Datos Bancarios
El sistema DEBE almacenar información bancaria para pagos de nómina.

- Banco (nombre)
- Número de cuenta
- Tipo de cuenta (ahorro/corriente)
- CCI (Código Interbancario)

### Requisito: EM-004 - Histórico de Empleados
El sistema DEBE mantener historial de empleados.

- Historial de contratos
- Historial de salarios
- Historial de cargos
- Estado (activo/inactivo/retirado)

---

## 2. Cálculo de Nómina (Payroll Calculation)

### Requisito: PR-001 - Estructura Salarial
El sistema DEBE soportar múltiples componentes salariales.

#### Componentes del ingreso:
- **Salario Base**: Monto principal (mensual/hora)
- **Horas Extra**: Tiempo adicional * tarifa hora extra
- **Bonificaciones**: Bonificaciones fixe/_variables
- **Comisiones**: Porcentaje sobre ventas/logros
- **Movilidad**: Bono de transporte
- **Alimentación**: Bono de alimentación
- **Otros**: Custom adicionales

#### Componentes de descuento:
- **AFP**: 10% del salario (variable por fondo)
- **ONP**: 13% del salario (opcional)
- **Essalud**: 9% del empregador
- **ITF**: 0.005% sobre depósitos > S/ 3500
- **Retención Judicial**: Si aplica
- **Otros descuentos**: Custom

### Requisito: PR-002 - Cálculo Automático de Nómina
El sistema DEBE calcular automáticamente la nómina mensual.

#### Escenario: Cálculo de nómina mensual
- GIVEN existen 5 empleados activos con contratos registrados
- WHEN el administrador ejecuta "Generar Nómina" para el mes actual
- THEN el sistema CALCULA para cada empleado:
  - Ingresos brutos (base + bonificaciones + horas extra)
  - Descuentos (AFP/ONP, Essalud, ITF, otros)
  - Ingresos netos
- AND GUARDA el registro de nómina
- AND MUESTRA resumen por empleado

#### Escenario: Cálculo con horas extra
- GIVEN un empleado tiene 10 horas extra registradas en el mes
- WHEN se calcula la nómina
- THEN el sistema APLICAR fórmula: (salario_base / 240) * 2 * horas_extra
- AND AGREGA al ingreso bruto

### Requisito: PR-003 - Esquemas de AFP
El sistema DEBE soportar diferentes AFPs con sus tasas.

| AFP | Aportación | Comisión | Total |
|-----|------------|-----------|-------|
| Prima | 10.00% | 1.25% | 11.25% |
| Habitat | 10.00% | 1.35% | 11.35% |
| Integra | 10.00% | 1.45% | 11.45% |
| Profuturo | 10.00% | 1.60% | 11.60% |

#### Escenario: Cambio de AFP
- GIVEN un empleado está en AFP Habitat
- WHEN el usuario cambia a AFP Prima
- THEN el sistema ACTUALIZA la AFP del empleado
- AND USA la nueva tasa en próximos cálculos

### Requisito: PR-004 - Periodo de Nómina
El sistema DEBE permitir configurar períodos de nómina.

- **Quincenal**: 15 y último día del mes
- **Mensual**: Último día del mes
- **Semanal**: Configurable

---

## 3. Generación de Recibos de Pago

### Requisito: RC-001 - Recibo de Pago en PDF
El sistema DEBE generar recibos de pago en formato PDF.

#### Estructura del recibo:
```
=========================================
         RECIBO DE PAGO DE NÓMINA
=========================================
Empresa: [Nombre de la empresa]
Periodo: [Mes/Año]
Empleado: [Nombre]
DNI: [Número]
Cargo: [Cargo]
Fecha de pago: [Fecha]

--- INGRESOS ---
Salario Base:        S/ X,XXX.XX
Horas Extra:         S/   XXX.XX
Bonificaciones:     S/   XXX.XX
Comisiones:         S/   XXX.XX
-----------------------
TOTAL INGRESOS:      S/ X,XXX.XX

--- DESCUENTOS ---
AFP (10%):           S/   XXX.XX
Essalud (9%):        S/   XXX.XX
ITF:                 S/     X.XX
Otros:               S/   XXX.XX
-----------------------
TOTAL DESCUENTOS:    S/   XXX.XX

=========================================
        PAGO NETO:        S/ X,XXX.XX
=========================================

Firma: _______________  Fecha: ________
```

#### Escenario: Generación de recibo individual
- GIVEN existe una nómina calculada para el empleado "Juan Pérez"
- WHEN el usuario selecciona "Generar Recibo" para ese empleado
- THEN el sistema GENERA un PDF con los datos del recibo
- AND permite DESCARGAR o IMPRIMIR

#### Escenario: Generación masiva de recibos
- GIVEN existe una nómina calculada para 5 empleados
- WHEN el usuario selecciona "Generar todos los recibos"
- THEN el sistema GENERA un PDF por cada empleado
- AND permite DESCARGAR como ZIP

### Requisito: RC-002 - Plantilla de Recibos
El sistema DEBE permitir personalizar la plantilla del recibo.

- Logo de la empresa
- Nombre de la empresa
- Colores (header, footer)
- Información de contacto

---

## 4. Reportes de Nómina

### Requisito: RP-001 - Resumen Mensual de Nómina
El sistema DEBE generar reportes mensuales de nómina.

#### Contenido del reporte:
- Total empleados activos
- Total nómina (bruto y neto)
- Total descuentos por tipo
- Comparación con mes anterior
- Distribución por departamento

#### Escenario: Generar reporte mensual
- GIVEN existe nómina calculada para "Marzo 2026"
- WHEN el usuario genera reporte mensual
- THEN el sistema MUESTRA:
  - Lista de empleados con detalle de ingresos/descuentos
  - Totales por tipo de ingreso
  - Totales por tipo de descuento
  - Total bruto, total descuentos, total neto

### Requisito: RP-002 - Reporte Anual de Nómina
El sistema DEBE generar reportes anuales.

#### Contenido:
- Resumen por mes
- Total anual por empleado
- Proporción de cargas sociales pagadas
- Constancia para declaracion de impuestos ( PDT - PLAME )

#### Escenario: Generar reporte anual
- GIVEN existen nóminas de todo el año 2025
- WHEN el usuario genera reporte anual 2025
- THEN el sistema MUESTRA:
  - Resumen mensual
  - Total anual por empleado
  - Total anual de cargas sociales

### Requisito: RP-003 - Exportación de Reportes
El sistema DEBE exportar reportes a Excel/PDF.

- Exportar a Excel (.xlsx)
- Exportar a PDF
- Exportar a CSV

---

## 5. Contabilidad General

### Requisito: CG-001 - Plan de Cuentas
El sistema DEBE mantener un plan de cuentas contable.

#### Estructura:
- 1xxx: Activos
- 2xxx: Pasivos
- 3xxx: Patrimonio
- 4xxx: Gastos
- 5xxx: Costos
- 6xxx: Ingresos

#### Escenario: Consulta de plan de cuentas
- GIVEN el usuario accede al módulo contable
- WHEN consulta el plan de cuentas
- THEN el sistema MUESTRA lista de cuentas con código, nombre, tipo

### Requisito: CG-002 - Asientos Contables
El sistema DEBE registrar asientos contables.

#### Estructura de asiento:
- Fecha
- Código de cuenta
- Descripción
- Debe/Haber
- Referencia

#### Escenario: Registro de asiento manual
- GIVEN el usuario registra un pago de alquiler
- WHEN ingresa: fecha, cuenta 401/411 (alquiler), monto, referencia
- THEN el sistema GUARDA el asiento
- AND actualiza libro diario

### Requisito: CG-003 - Generación Automática de Asientos
El sistema DEBE generar asientos automáticamente desde nómina.

#### Escenario: Asiento automático de nómina
- GIVEN se calcula la nómina mensual
- WHEN se confirma el cálculo
- THEN el sistema GENERA asientos:
  - 621 Sueldos (Debe) → 401 Beneficios sociales (Haber)
  - 627 Seguridad social (Debe) → 401 Beneficios sociales (Haber)
  - 401 Remuneraciones por pagar (Debe) → 104 Banco (Haber)

### Requisito: CG-004 - Libro Diario
El sistema DEBE mantener libro diario de asientos.

- Registro cronológico de todos los asientos
- Filtrado por rango de fechas
- Búsqueda por descripción o cuenta
- Exportación a Excel

---

## 6. Facturación e Ingresos

### Requisito: FA-001 - Facturas de Servicios Educativos
El sistema DEBE generar facturas por servicios educativos.

#### Datos de factura:
- Serie y número correlativo
- Fecha de emisión
- Datos del cliente (razón social, RUC)
- Detalle de servicios
- Subtotal, IGV (18%), total
- Forma de pago

#### Escenario: Generar factura por mensualidad
- GIVEN un estudiante tiene contrato de mensualidad
- WHEN se genera la mensualidad de marzo
- THEN el sistema CREA una factura en estado "pendiente"
- AND puede GENERAR PDF
- AND puede ENVIAR por email (futuro)

### Requisito: FA-002 - Notas de Crédito/Débito
El sistema DEBE permitir emitir notas de crédito y débito.

- Por anulación total/parcial de factura
- Por descuento/aplicación de intereses

### Requisito: FA-003 - Registro de Pagos
El sistema DEBE registrar pagos de facturas.

- Pago parcial o total
- Métodos: efectivo, transferencia, tarjeta
- Fecha de pago
- Referencia bancaria

---

## 7. Reportes Contables

### Requisito: RC-001 - Balance de Comprobación
El sistema DEBE generar balance de comprobación.

- Listado de cuentas con saldo deudor/acredor
- Totales deben coincidir
- Fecha de corte configurable

### Requisito: RC-002 - Estado de Resultados
El sistema DEBE generar estado de resultados.

- Ingresos por periodo
- Gastos por periodo
- Utilidad o pérdida

### Requisito: RC-003 - Balance General
El sistema DEBE generar balance general.

- Activos, pasivos, patrimonio
- Estado de situación financiera

---

## 8. Integración y Configuración

### Requisito: CF-001 - Configuración de Empresa
El sistema DEBE permitir configurar datos de la empresa.

- Razón social
- RUC
- Dirección
- Logo
- Representante legal

### Requisito: CF-002 - Configuración Contable
El sistema DEBE permitir configurar parámetros contables.

- Moneda (Soles/Dólares)
- Periodo contable
- Tipos de cambio (si aplica)

### Requisito: CF-003 - Respaldo de Datos
El sistema DEBE realizar respaldo de base de datos.

- Exportar a archivo .db
- Importar desde archivo

---

## Escenarios de Edge Cases

### Edge Case: Empleado con múltiples contratos
- GIVEN un empleado tiene contrato por horas Y prestación de servicios
- WHEN se calcula nómina
- THEN el sistema SEPARA cálculo por cada contrato
- AND genera recibo por cada uno

### Edge Case: Nómina con empleado cesado
- GIVEN un empleado fue cesado el día 15 del mes
- WHEN se calcula nómina
- THEN el sistema PROPORCIONALIZA el salario (15/30 días)
- AND aplica liquidación si corresponde

### Edge Case: Mes con días no laborables
- GIVEN el mes tiene feriados legales
- WHEN se calculan horas extra
- THEN el sistema APLICAR tarifa doble para feriados

### Edge Case: Descuento por embargo judicial
- GIVEN un empleado tiene embargo judicial de S/ 500
- WHEN se calcula nómina
- THEN el sistema APLICAR descuento hasta completar embargo
- AND registrar historial

---

## Definiciones

| Término | Definición |
|---------|------------|
| AFP | Administradora de Fondos de Pensiones |
| Essalud | Seguro Social de Salud del Perú |
| ONP | Oficina de Normalización Previsional |
| ITF | Impuesto a las Transacciones Financieras |
| IGV | Impuesto General a las Ventas (18%) |
| PDT | PDT - Plame: Programa de Declaración Telemática |
| Salario Base | Remuneración mensual/hora sin adicionales |
| Ingreso Bruto | Total de ingresos antes de descuentos |
| Ingreso Neto | Total a pagar después de descuentos |
| CCI | Código de Cuenta Interbancaria |

---

## Métricas de Éxito

- Cálculo de nómina 100% automático
- Precisión en descuentos legales (AFP, Essalud)
- Generación de PDF de recibos en < 2 segundos
- Reporte mensual disponible el 1er día del siguiente mes
- Exportación de reportes a Excel/PDF sin pérdida de formato