use std::marker::PhantomData;

use serde::{
    ser::Serialize,
    de::DeserializeOwned,
};
use serde_json;
use drop_bomb::DropBomb;

use ::{
    Result,
    req::{ClientRequest, Notification},
    io::{RawResponse, RawRequest, RawNotification},
};

pub struct Responder<R: ClientRequest> {
    id: u64,
    bomb: DropBomb,
    ph: PhantomData<fn(R)>,
}

impl<R: ClientRequest> Responder<R> {
    pub fn into_response(mut self, result: Result<R::Result>) -> Result<RawResponse> {
        self.bomb.defuse();
        let res = match result {
            Ok(result) => {
                RawResponse {
                    id: self.id,
                    result: serde_json::to_value(result)?,
                    error: serde_json::Value::Null,
                }
            }
            Err(e) => {
                error_response(
                    self.id,
                    ErrorCode::InternalError,
                    format!("internal error: {}", e),
                )?
            }
        };
        Ok(res)
    }
}

fn parse_request_as<R: ClientRequest>(raw: RawRequest)
                                      -> Result<::std::result::Result<(R::Params, Responder<R>), RawRequest>>
{
    if raw.method != R::METHOD {
        return Ok(Err(raw));
    }

    let params: R::Params = serde_json::from_value(raw.params)?;
    let responder = Responder {
        id: raw.id,
        bomb: DropBomb::new("dropped request"),
        ph: PhantomData,
    };
    Ok(Ok((params, responder)))
}

pub fn handle_request<R, F>(req: &mut Option<RawRequest>, f: F) -> Result<()>
    where
        R: ClientRequest,
        F: FnOnce(R::Params, Responder<R>) -> Result<()>
{
    match req.take() {
        None => Ok(()),
        Some(r) => match parse_request_as::<R>(r)? {
            Ok((params, responder)) => f(params, responder),
            Err(r) => {
                *req = Some(r);
                Ok(())
            }
        }
    }
}

fn parse_notification_as<N>(raw: RawNotification) -> Result<::std::result::Result<N::Params, RawNotification>>
    where
        N: Notification,
        N::Params: DeserializeOwned,
{
    if raw.method != N::METHOD {
        return Ok(Err(raw));
    }
    let params: N::Params = serde_json::from_value(raw.params)?;
    Ok(Ok(params))
}

pub fn handle_notification<N, F>(not: &mut Option<RawNotification>, f: F) -> Result<()>
    where
        N: Notification,
        N::Params: DeserializeOwned,
        F: FnOnce(N::Params) -> Result<()>
{
    match not.take() {
        None => Ok(()),
        Some(n) => match parse_notification_as::<N>(n)? {
            Ok(params) => f(params),
            Err(n) => {
                *not = Some(n);
                Ok(())
            }
        }
    }
}

pub fn send_notification<N>(params: N::Params) -> RawNotification
    where
        N: Notification,
        N::Params: Serialize
{
    RawNotification {
        method: N::METHOD.to_string(),
        params: serde_json::to_value(params)
            .unwrap(),
    }
}

pub fn unknown_method(id: u64) -> Result<RawResponse> {
    error_response(id, ErrorCode::MethodNotFound, "unknown method")
}

fn error_response(id: u64, code: ErrorCode, message: impl Into<String>) -> Result<RawResponse> {
    #[derive(Serialize)]
    struct Error {
        code: i32,
        message: String,
    }
    let resp = RawResponse {
        id,
        result: serde_json::Value::Null,
        error: serde_json::to_value(Error {
            code: code as i32,
            message: message.into(),
        })?,
    };
    Ok(resp)
}

#[allow(unused)]
enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerErrorStart = -32099,
    ServerErrorEnd = -32000,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestCancelled = -32800,
}
