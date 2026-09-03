use serde::Deserialize;

/// Exactly what `api.esv.org/v3/passage/text/` sends back.
///
/// Deliberately `pub(crate)`: this is ESV's shape, not coer's. Letting it out
/// of this crate would put a third party's JSON layout into our domain, and
/// their next format change would become our problem everywhere at once.
#[derive(Deserialize, Debug)]
pub(crate) struct EsvResponse {
    pub canonical: String,
    pub passages: Vec<String>,
    // Ignoring PassageMeta, which carries chapter start/end and prev/next verse.
}
