# services/ — Capa de Orquestación

Conecta la lógica de dominio (`domain/`) con la base de datos (SeaORM) y produce las exportaciones (PDF, XML, TXT). Cada service corresponde a un recurso de la API.

Los handlers en `api/` siempre delegan a un service — nunca acceden a la BD directamente.

## Archivos

| Archivo                       | Descripción                                                              |
|-------------------------------|--------------------------------------------------------------------------|
| `invoice_service.rs`          | Crear, listar y obtener facturas (inmutables una vez emitidas).          |
| `credit_note_service.rs`      | Crear, listar y obtener notas de crédito.                                |
| `debit_note_service.rs`       | Crear, listar y obtener notas de débito.                                 |
| `delivery_note_service.rs`    | Crear, listar y obtener guías de despacho.                               |
| `client_service.rs`           | CRUD de clientes con validación de RIF.                                  |
| `company_service.rs`          | Obtener y actualizar el perfil fiscal de la empresa.                     |
| `book_service.rs`             | Consulta y registro de libros de compras y ventas.                       |
| `withholding_iva_service.rs`  | Crear, listar y obtener retenciones IVA.                                 |
| `withholding_islr_service.rs` | Crear, listar y obtener retenciones ISLR.                                |
| `pdf_service.rs`              | Generación de PDFs con `genpdf` (facturas, notas, comprobantes).         |
| `xml_service.rs`              | XML quincenal de retenciones IVA en formato SENIAT.                      |
| `txt_service.rs`              | TXT mensual de retenciones ISLR en formato SENIAT.                       |
| `export_service.rs`           | Orquesta la generación de exportaciones combinando los services anteriores. |
| `seeder.rs`                   | Seed inicial: crea el perfil de empresa y el rango de números de control.|

---

## invoice_service.rs

### `create_invoice(db, request, user_id, company_id) -> Result<InvoiceResponse>`

1. Valida el request con las reglas del dominio (`domain::invoice::validation`).
2. Obtiene la tasa de cambio BCV si la moneda es USD.
3. Usa `InvoiceBuilder` para calcular totales fiscales.
4. Asigna el siguiente número de control del rango activo (en transacción).
5. Asigna el siguiente número de secuencia de la serie.
6. Persiste la factura e ítems en una transacción.
7. Registra en `audit_logs`.

Las facturas son **inmutables** una vez emitidas. No existe endpoint de actualización ni de anulación.
Para corregir una factura emitida se debe emitir una **Nota de Crédito** que la referencie
por número de control y número de factura (PA SNAT/2011/0071).

### `list_invoices(db, filters) -> Result<PaginatedResponse<InvoiceRow>>`

Filtros disponibles: `from`, `to` (fechas), `client_id`, `status`, `page`, `per_page`.

---

## client_service.rs

### `create_client(db, request, user_id) -> Result<ClientResponse>`

Valida el RIF si se provee (formato y dígito verificador). Los clientes sin RIF se marcan automáticamente como consumidores finales, lo que activa el texto obligatorio "SIN DERECHO A CRÉDITO FISCAL" en sus facturas.

---

## book_service.rs

### `get_purchase_book(db, filters) -> Result<PurchaseBookResponse>`

Consolida todas las compras del período (`period=YYYY-MM`) con los totales por tipo de IVA.

### `get_sales_book(db, filters) -> Result<SalesBookResponse>`

Consolida todas las ventas del período con los totales por tipo de IVA. Incluye resumen diario.

---

## pdf_service.rs

El service de mayor tamaño (~1600 líneas). Genera PDFs usando `genpdf` con el formato exigido por las providencias SENIAT.

### Documentos soportados

| Función                         | Documento generado                                    |
|---------------------------------|-------------------------------------------------------|
| `generate_invoice_pdf`          | Factura con datos fiscales completos.                 |
| `generate_credit_note_pdf`      | Nota de crédito con referencia a factura original.    |
| `generate_debit_note_pdf`       | Nota de débito con referencia a factura original.     |
| `generate_iva_withholding_pdf`  | Comprobante de retención IVA.                         |
| `generate_islr_withholding_pdf` | Comprobante ARC ISLR.                                 |

Los PDFs incluyen:
- Datos del emisor (razón social, RIF, domicilio fiscal, teléfono).
- Datos del receptor (razón social/nombre, RIF o cédula).
- Detalle de ítems con descripción, cantidad, precio unitario, IVA.
- Totales: subtotal, IVA desglosado por tasa, total Bs., total USD.
- Número de control, número de factura, fecha.
- Texto "SIN DERECHO A CRÉDITO FISCAL" cuando aplica.

---

## xml_service.rs

Genera el archivo XML quincenal de retenciones IVA en el formato requerido para carga en el portal SENIAT.

### `generate_iva_xml(db, period) -> Result<String>`

El `period` tiene formato `YYYYMM-QQ` donde `QQ` es `01` (primera quincena) o `02` (segunda quincena).

```
Ejemplo: period=202603-01  → primera quincena de marzo 2026
         period=202603-02  → segunda quincena de marzo 2026
```

---

## txt_service.rs

Genera el archivo TXT mensual de retenciones ISLR en el formato del sistema ARC (Agente de Retención en la Fuente) del SENIAT.

### `generate_islr_txt(db, period) -> Result<String>`

El `period` tiene formato `YYYY-MM`. Incluye todas las retenciones ISLR del mes con los totales por código de actividad.

---

## seeder.rs

### `run_seed(db, seed_config) -> Result<()>`

Ejecutado solo una vez en el primer arranque si las variables `SEED_*` están definidas. Crea:

1. El `company_profile` con los datos fiscales de la empresa.
2. El `numbering_sequence` para la serie de facturas.
3. El `control_number_range` con el rango autorizado por la imprenta SENIAT.

Si ya existe un perfil activo, el seed no hace nada (idempotente).
