pub mod helpers;
pub mod books;
pub mod clients;
pub mod company;
pub mod credit_notes;
pub mod debit_notes;
pub mod delivery_notes;
pub mod invoices;
pub mod reports;
pub mod withholdings;

use actix_web::web;

/// Configura todas las rutas de la API bajo /billingVE/v1/
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/billingVE/v1")
            .configure(invoices::configure)
            .configure(credit_notes::configure)
            .configure(debit_notes::configure)
            .configure(clients::configure)
            .configure(company::configure)
            .configure(books::configure)
            .configure(withholdings::configure)
            .configure(delivery_notes::configure)
            .configure(reports::configure),
    );
}
