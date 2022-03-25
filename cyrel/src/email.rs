use askama::Template;
use lettre::{
    message::{MultiPart, SinglePart},
    Message,
};

use crate::SETTINGS;

fn gen<T, H>(email: &str, subject: &str, txt: T, html: H) -> anyhow::Result<Message>
where
    T: Template,
    H: Template,
{
    let parts = MultiPart::alternative()
        .singlepart(SinglePart::plain(txt.render()?))
        .singlepart(SinglePart::html(html.render()?));

    let msg = Message::builder()
        .from(SETTINGS.smtp.from.parse()?)
        .to(email.parse()?)
        .subject(subject)
        .multipart(parts)?;

    Ok(msg)
}

#[derive(Template)]
#[template(path = "inscription.txt")]
struct InscriptionTxtTemplate<'a> {
    hash: &'a str,
}

#[derive(Template)]
#[template(path = "inscription.html")]
struct InscriptionHtmlTemplate<'a> {
    hash: &'a str,
}

pub fn gen_inscription(email: &str, hash: &str) -> anyhow::Result<Message> {
    gen(
        email,
        "Inscription",
        InscriptionTxtTemplate { hash },
        InscriptionHtmlTemplate { hash },
    )
}

#[derive(Template)]
#[template(path = "reset.txt")]
struct ResetTxtTemplate<'a> {
    firstname: &'a str,
    lastname: &'a str,
    hash: &'a str,
}

#[derive(Template)]
#[template(path = "reset.html")]
struct ResetHtmlTemplate<'a> {
    firstname: &'a str,
    lastname: &'a str,
    hash: &'a str,
}

pub fn gen_reset(
    email: &str,
    firstname: &str,
    lastname: &str,
    hash: &str,
) -> anyhow::Result<Message> {
    gen(
        email,
        "Réinitialisation du mot de passe",
        ResetTxtTemplate {
            firstname,
            lastname,
            hash,
        },
        ResetHtmlTemplate {
            firstname,
            lastname,
            hash,
        },
    )
}
