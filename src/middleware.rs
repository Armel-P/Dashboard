use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
    error::{ErrorUnauthorized, ErrorInternalServerError},
    web, Error, HttpMessage
};
use jsonwebtoken::DecodingKey;
use uuid::Uuid;

use crate::{
    password::verify_jwt,
    structs::encrypt::{Claims}
};

pub async fn jwt_auth<B: MessageBody + 'static>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth_header = match req.headers().get(header::AUTHORIZATION) {
        Some(header) => {
            match header.to_str() {
                Ok(auth_header) => auth_header,
                Err(_) => return Err(ErrorUnauthorized("Wrong header format"))
            }
        },
        None => return Err(ErrorUnauthorized("Missing header"))
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorUnauthorized("Missing Bearer token"));
    }
    let token = &auth_header[7..];
 
    let secret = match req.app_data::<web::Data<Vec<u8>>>() {
        Some(secret) => secret,
        None => return Err(ErrorInternalServerError("Secret fetch fail"))
    };
 
    let claims: Claims = match verify_jwt(token, &DecodingKey::from_secret(secret.as_ref())) {
        Ok(claims) => claims,
        Err(_) => return Err(ErrorUnauthorized("Wrong JWT"))
    };

    let token_id = match Uuid::parse_str(claims.sub.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => return Err(ErrorUnauthorized("Wrong JWT"))
    };
 
    req.extensions_mut().insert(token_id);
    next.call(req).await
}
