use crate::firefly::firefly_server::Firefly;
use crate::firefly::{Ack, Empty, Gradient, Led as ProtoLed, Rgb, State as ProtoState, White};
use crate::led::Led;
use crate::strip::Strip;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};

#[derive(Debug)]
pub struct FireflyService {}

#[tonic::async_trait]
impl Firefly for FireflyService {
    async fn on(&self, request: Request<Empty>) -> Result<Response<Ack>, Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;
        state
            .strip
            .on()
            .map_err(|_| Status::new(Code::Internal, "Failed setting strip to on."))?;
        Ok(Response::new(Ack { status: true }))
    }

    async fn off(&self, request: Request<Empty>) -> Result<Response<Ack>, Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;
        state
            .strip
            .off()
            .map_err(|_| Status::new(Code::Internal, "Failed setting strip to off."))?;
        Ok(Response::new(Ack { status: true }))
    }

    async fn set_rgb(&self, request: Request<Rgb>) -> Result<Response<Ack>, Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;

        let rgb = request.get_ref();

        state
            .strip
            .fill(Led::from_rgb(
                rgb.red as u8,
                rgb.green as u8,
                rgb.blue as u8,
            ))
            .map_err(|_| Status::new(Code::Internal, "Failed filling strip."))?;
        Ok(Response::new(Ack { status: true }))
    }

    async fn set_white(&self, request: Request<White>) -> Result<Response<Ack>, Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;

        let white = request.get_ref();

        state
            .strip
            .fill(Led::from_w(white.white as u8))
            .map_err(|_| Status::new(Code::Internal, "Failed filling strip."))?;
        Ok(Response::new(Ack { status: true }))
    }

    async fn set_gradient(&self, request: Request<Gradient>) -> Result<Response<Ack>, Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;

        let gradient = request
            .get_ref()
            .colors
            .iter()
            .map(|color| (color.red as u8, color.green as u8, color.blue as u8))
            .collect();

        state
            .strip
            .set_gradient(gradient)
            .map_err(|_| Status::new(Code::Internal, "Failed setting gradient."))?;
        Ok(Response::new(Ack { status: true }))
    }

    async fn get_state(&self, request: Request<Empty>) -> Result<Response<ProtoState>, Status> {
        let state = request
            .extensions()
            .get::<Arc<Mutex<FireFlyState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strip."))?
            .lock()
            .await;

        Ok(Response::new(ProtoState {
            on: state.strip.is_on(),
            leds: state
                .strip
                .leds
                .iter()
                .map(|led| ProtoLed {
                    red: led.r as i32,
                    green: led.g as i32,
                    blue: led.b as i32,
                    white: led.w as i32,
                })
                .collect(),
        }))
    }
}

pub struct FireFlyState {
    pub strip: Strip,
}
