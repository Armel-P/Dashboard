use lettre::{
    message::header::ContentType,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl Mailer {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host = std::env::var("SMTP_HOST")?;
        let from = std::env::var("SMTP_FROM")?;
        let port: u16 = std::env::var("SMTP_PORT")?.parse()?;

        let transport =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
                .port(port)
                .build();

        Ok(Self { transport, from })
    }

    pub async fn send_verification(&self, to: &str, link: &str) -> Result<(), Box<dyn std::error::Error>> {
        let email = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject("Confirm your account")
            .header(ContentType::TEXT_PLAIN)
            .body(format!("Click to confirm your account (valid 5 min):\n\n{link}"))?;

        self.transport.send(email).await?;
        Ok(())
    }
}
