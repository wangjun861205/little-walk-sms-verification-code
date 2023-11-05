use actix_web::{
    error::ErrorInternalServerError,
    web::{Data, Json, Path},
    Error, HttpResponse,
};
use serde::Serialize;
use sms_verification_code_service::core::{
    generator::Generator, sender::Sender, service::Service, store::Store,
};

pub async fn send_code<SD, ST, GN>(
    service: Data<Service<SD, ST, GN>>,
    phone: Path<(String,)>,
) -> HttpResponse
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    match service
        .acquire_executor(&phone.0.to_owned())
        .await
        .lock()
        .await
        .send_code()
        .await
    {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(_) => HttpResponse::Ok().finish(),
    }
}

#[derive(Debug, Serialize)]
pub struct VerifyCodeResp {
    pub is_ok: bool,
}

pub async fn verify_code<SD, ST, GN>(
    service: Data<Service<SD, ST, GN>>,
    phone_and_code: Path<(String, String)>,
) -> Result<Json<VerifyCodeResp>, Error>
where
    SD: Sender + Clone,
    ST: Store + Clone,
    GN: Generator + Clone,
{
    let (phone, code) = (phone_and_code.0.to_owned(), phone_and_code.1.to_owned());
    match service
        .acquire_executor(&phone)
        .await
        .lock()
        .await
        .verify_code(&code)
        .await
    {
        Err(err) => Err(ErrorInternalServerError(err.to_string())),
        Ok(()) => Ok(Json(VerifyCodeResp { is_ok: true })),
    }
}
