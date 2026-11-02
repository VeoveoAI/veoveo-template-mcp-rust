//! Embeds the skills under `skills/`, which the server serves at `skill://template/…`.
fn main() {
    veoveo_skills::build::embed_skills("skills").expect("the template's skills");
}
