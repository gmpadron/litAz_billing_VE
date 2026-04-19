//! Funciones compartidas para los handlers de la API.
//!
//! Para extraer el usuario autenticado, los handlers deben usar el extractor
//! `AuthenticatedUser` (definido en `middleware/auth.rs`) como parámetro,
//! en lugar de funciones helper.
