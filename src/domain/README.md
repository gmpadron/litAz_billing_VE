# domain/ — Lógica de Negocio

Capa de dominio pura: sin I/O, sin Actix Web, sin SeaORM. Solo cálculos, validaciones y transformaciones en Rust + `rust_decimal`. Esto la hace completamente testeable sin base de datos.

## Sub-módulos

| Módulo         | Descripción                                                                        |
|----------------|------------------------------------------------------------------------------------|
| `invoice/`     | Builder de facturas, gestión de estados, validación fiscal SENIAT.                 |
| `numbering/`   | Validación de RIF venezolano, números de control, secuencias de numeración.        |
| `tax/`         | Cálculos de IVA (tasas múltiples) e ISLR (Decreto 1.808).                         |
| `withholding/` | Cálculos de retenciones IVA e ISLR.                                                |
| `books/`       | Consolidación de libros de compras/ventas, resumen diario.                         |
| `currency/`    | Consulta de tasa de cambio BCV desde API externa.                                  |

---

## invoice/

### `builder.rs` — InvoiceBuilder

Construye el modelo de factura a partir de un `CreateInvoiceRequest`. Aplica las reglas fiscales:

- Asigna número de control desde el rango activo.
- Calcula subtotal, IVA (por tasa), total en Bs. y USD.
- Determina si aplica "SIN DERECHO A CRÉDITO FISCAL" (cliente sin RIF).
- Genera el número de factura con la secuencia correspondiente.

### `credit_note.rs` y `debit_note.rs`

Construyen notas de crédito/débito validando que:
- La factura referenciada exista y esté en estado activo.
- El monto no supere el de la factura original.
- Se preserve la tasa de cambio de la factura original (no la del día).

### `status.rs` — Estados de factura

```
Borrador → Emitida   (única transición válida)
```

Las facturas son **inmutables** una vez emitidas (PA SNAT/2011/0071).
No existe transición a "Anulada". Para corregir una factura emitida
se emite una Nota de Crédito que la referencia. Las facturas nunca se eliminan.

### `validation.rs`

Valida las reglas de negocio antes de persistir:
- RIF del cliente en formato correcto (si se provee).
- Cantidades y precios positivos.
- La tasa de cambio es obligatoria si la moneda es USD.
- El número de control pertenece al rango activo y no está duplicado.

---

## numbering/

### `rif.rs` — Validación de RIF venezolano

Implementa el algoritmo del dígito verificador SENIAT:

```rust
// Tipos de RIF soportados
pub enum RifType { J, V, E, G, P, C }

// Valida formato y dígito verificador
pub fn validate_rif(rif: &str) -> Result<ParsedRif, RifError>

// Ejemplos válidos:
// "J-12345678-9"
// "V-12345678-9"
// "E-12345678-9"
```

### `control_number.rs` — Números de control SENIAT

Gestiona los rangos autorizados por imprenta. El número de control tiene formato `00-XXXXXXXX` (prefijo + 8 dígitos). Al crear una factura se toma el siguiente número del rango activo y se valida que no esté agotado.

### `invoice_sequence.rs` — Secuencias de numeración

Genera números de factura correlativos. Cada serie (ej. `0001`) tiene su propia secuencia. Los números son inmutables una vez asignados.

---

## tax/

### `iva.rs` — Cálculo de IVA

Calcula el impuesto por cada línea de factura y el total consolidado. Soporta facturas con tasas múltiples (ej. ítems al 16% y al 8% en la misma factura).

```rust
pub struct IvaTotals {
    pub base_imponible_general: Decimal,   // Base gravable al 16%
    pub iva_general: Decimal,              // IVA al 16%
    pub base_imponible_reducida: Decimal,  // Base gravable al 8%
    pub iva_reducida: Decimal,             // IVA al 8%
    pub base_imponible_adicional: Decimal, // Base gravable al 31% (lujo)
    pub iva_adicional: Decimal,            // IVA al 31%
    pub exento: Decimal,                   // Monto exento
    pub total_iva: Decimal,
    pub total_con_iva: Decimal,
}
```

Todos los valores usan `rust_decimal` (128-bit) para precisión exacta. No se usa `f64`.

### `islr.rs` — Cálculo de ISLR

Calcula la base imponible y el monto de retención ISLR según el Decreto 1.808. Las tasas varían por código de actividad económica.

---

## withholding/

### `iva_wh.rs` — Retención IVA

Los contribuyentes especiales retienen el 75% o el 100% del IVA al proveedor. Genera el comprobante con:
- Número de comprobante correlativo.
- Período quincenal (primera o segunda quincena).
- Datos del agente de retención y del proveedor.

### `islr_wh.rs` — Retención ISLR

Calcula la retención ISLR según el código de actividad. Genera el comprobante ARC (Agente de Retención Comprobante) mensual.

---

## books/

### `sales_book.rs` y `purchase_book.rs`

Consolidan las operaciones del período en el formato requerido por el SENIAT:
- Agrupadas por fecha de operación.
- Columnas separadas por tipo de IVA (general, reducido, adicional, exento).
- Totales del período.

### `daily_summary.rs`

Genera el resumen diario de operaciones para el libro, sumando todas las facturas del día.

---

## currency/

### `bcv.rs` — Tasa de cambio BCV

Consulta la API del Banco Central de Venezuela para obtener la tasa de cambio oficial del día. Si la consulta falla (timeout, API caída), retorna error — no usa tasa aproximada.

La tasa se almacena como snapshot inmutable en cada documento en el momento de su creación. Las consultas posteriores al mismo documento siempre usan la tasa original, no la actual.

```rust
pub async fn fetch_bcv_rate() -> Result<Decimal, CurrencyError>
```
