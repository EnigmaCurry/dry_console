use anyhow::Result;
use rusty_paseto::prelude::*;
use time::format_description::well_known::Rfc3339;

pub fn generate_token(
    secret: &[u8],
    expiration_minutes: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let expiration_time = (time::OffsetDateTime::now_utc()
        + time::Duration::minutes(expiration_minutes))
    .format(&Rfc3339)?;
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(secret));
    let token = PasetoBuilder::<V4, Local>::default()
        .set_claim(ExpirationClaim::try_from(expiration_time)?)
        .build(&key)?;
    Ok(token)
}

pub fn validate_token(token: &str, secret: &[u8]) -> Result<bool> {
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(secret));
    let _parsed = PasetoParser::<V4, Local>::default().parse(&token, &key)?;
    Ok(true)
}
