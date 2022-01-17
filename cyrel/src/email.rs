extern crate lettre;

use self::lettre::message::{header, MultiPart, SinglePart};
use self::lettre::transport::smtp::response::Response;
use crate::SETTINGS;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tracing::{error, info, warn};

pub struct Email {}

impl Email {
    pub fn send_verification_email(email: String, department: String, hash: String) -> Response {
        let mut body_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Inscription sur Cyrel</title>
</head>
<body>
    <h1>Inscription sur Cyrel</h1>
    <p>Merci de vous inscrire sur Cyrel. Il vous manque quelques étapes afin de finaliser votre inscription.</p>
    <p>Copier-coller le code sur la page d'inscription pour continuer :
        <b>
"#.to_string();
        body_html.push_str(&hash.to_owned());
        body_html.push_str(
            r#"
        </b>
       </p>
       </body>
</html>
        "#,
        );
        let mut body_text = "Inscription sur Cyrel : \
         Merci de vous inscrire sur Cyrel. Il vous manque quelques étapes afin de finaliser votre inscription.  \
         Copier-coller le code sur la page d'inscription pour continuer :  \
        ".to_string();
        body_text.push_str(&hash.to_owned());
        let email = Message::builder()
            .from(SETTINGS.smtp.from.as_str().parse().unwrap())
            .to(email.parse().unwrap())
            .subject("Inscription")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(body_text), // Every message should have a plain text fallback.
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body_html),
                    ),
            )
            .unwrap();

        let creds = Credentials::new(
            SETTINGS.smtp.username.to_string(),
            SETTINGS.smtp.password.to_string(),
        );

        // Open a local connection on port 25
        let mut mailer = SmtpTransport::relay(SETTINGS.smtp.server.as_str()) //TODO Maybe it's better to init mailer one time for all
            .unwrap()
            .credentials(creds)
            .build();
        // Send the email
        let result = mailer.send(&email);

        if result.is_ok() {
            info!("Email sent");
        } else {
            warn!("Could not send email: {:?}", result);
        }

        result.unwrap()
    }
}
